//! The device loop feeding [`crate::SharedState`] - mirrors the
//! headless modes' engine loop, plus the draw-side writes.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use tangible_objects_engine::Engine;
use tangible_objects_engine::RecordingSession;
use tangible_objects_engine::TouchDevice;
use tangible_objects_engine::TouchState;
use tangible_objects_model::ContactCount;
use tangible_objects_model::DetectionConfig;
use tangible_objects_model::DisplayRotation;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameNumber;
use tangible_objects_model::Milliseconds;
use tangible_objects_model::PatternBuf;
use tangible_objects_model::ProfileInfo;
use tangible_objects_model::TouchSet;
use tangible_objects_runtime::DeviceInfo;
use tangible_objects_runtime::EngineRuntime;

use crate::GuiHooks;
use crate::background_alpha::BackgroundAlpha;
use crate::calibration_source::CalibrationSource;
use crate::field_set_row::FieldSetRow;
use crate::language_loader;
use crate::shared_state::SharedState;
use crate::shared_state::mode_is_live;
use i18n_embed_fl::fl;
use tracing::error;
use tracing::info;

/// Consecutive same-size frames before a point set counts as stable
/// for profile capture.
pub(crate) const STABLE_THRESHOLD: u32 = 10;

/// Field-set card rows for `SharedState` - set name, registration
/// owner, enabled flag and field count per registered set.
fn field_set_rows(engine: &Engine) -> Vec<FieldSetRow> {
    engine
        .field_sets()
        .iter()
        .map(|set| FieldSetRow {
            name: set.name.clone(),
            owner: engine.field_set_owner(&set.name).cloned(),
            enabled: set.enabled,
            field_count: set.fields.len(),
        })
        .collect()
}

/// Device loop thread: opens the evdev device, runs the engine per
/// `SYN_REPORT` frame, publishes snapshots/events through the runtime
/// and writes the GUI's [`SharedState`]. Daemon semantics (interfaces,
/// control commands, recording) arrive via [`GuiHooks`].
/// Virtual suppression/forwarding counts - what the filter proxy
/// would do to this frame under `mode`. The GUI has no proxy; the
/// counters show ingest-filtered plus mode-suppressed contacts so the
/// status line matches `--filter` semantics.
fn virtual_suppression(
    engine: &Engine,
    mode: FilterMode,
    touches: TouchSet<'_>,
) -> (ContactCount, ContactCount) {
    let total = touches.len() as u32;
    let suppressed = match mode {
        FilterMode::Off => 0,
        FilterMode::Immediate | FilterMode::Gated | FilterMode::Guarded => {
            let bound = engine.bound_slots();
            let field = engine.field_suppressed_slots(touches);
            touches
                .slots_where(|t| bound.contains(&t.slot) || field.contains(&t.slot))
                .len() as u32
        }
        FilterMode::TangiblesOnly => total,
    };
    (suppressed.into(), (total - suppressed).into())
}

pub fn evdev_thread(
    device_path: String,
    patterns: PatternBuf,
    state: Arc<Mutex<SharedState>>,
    config: DetectionConfig,
    rotation: DisplayRotation,
    device_profiles: Option<Arc<PathBuf>>,
    hooks: GuiHooks,
) {
    let mut device = match TouchDevice::open(std::path::Path::new(&device_path)) {
        Ok(d) => d,
        Err(e) => {
            if let Ok(mut s) = state.lock() {
                s.device_error = Some(fl!(
                    language_loader(),
                    "device-error",
                    error = e.to_string()
                ));
            }
            return;
        }
    };
    if config.import_slots {
        device.queue_state_import();
    }

    let mut engine = Engine::new(patterns);
    if let Some(mm) = config.edge {
        engine.set_edge_margin(mm);
    }
    let (
        dead_zones_visible,
        spells_visible,
        background_alpha,
        edge_margin_visible,
        fields_visible,
        ruler_visible,
        positions_visible,
        filter_mode,
        touch_passthrough,
    ) = state
        .lock()
        .map(|s| {
            (
                s.dead_zones_visible,
                s.spells_visible,
                s.background_alpha.percent(),
                s.edge_margin_visible,
                s.fields_visible,
                s.ruler_visible,
                s.positions_visible,
                s.filter_mode,
                s.touch_passthrough,
            )
        })
        .unwrap_or_default();
    engine.set_fields(config.fields);
    engine.set_dead_zones(config.dead_zones);
    engine.set_completion(config.completion);

    // Event hub for interface crates (GraphQL/D-Bus/MCP) - the GUI runs
    // its own device loop, so it owns the runtime side like the
    // headless modes do.
    let profiles: Vec<ProfileInfo> = engine.patterns().iter().map(ProfileInfo::from).collect();
    let (mut runtime, handle) = EngineRuntime::new(Vec::new(), profiles);
    runtime.set_completion(config.completion);
    runtime.set_display_rotation(rotation);
    runtime.set_field_sets(engine.field_sets().to_vec());
    runtime.set_dead_zones(engine.dead_zones().to_vec());
    if let Ok(mut s) = state.lock() {
        s.command_handle = Some(handle.clone());
    }
    runtime.set_dead_zones_visible(dead_zones_visible);
    runtime.set_spells_visible(spells_visible);
    runtime.set_background_alpha(background_alpha.into());
    runtime.set_edge_margin_visible(edge_margin_visible);
    runtime.set_fields_visible(fields_visible);
    runtime.set_ruler_visible(ruler_visible);
    runtime.set_positions_visible(positions_visible);
    runtime.set_filter_mode(filter_mode);
    runtime.set_touch_passthrough(touch_passthrough);
    (hooks.spawn_interfaces)(&handle);

    let mut touch_state = TouchState::new();
    if let Some(cal) =
        (hooks.effective_calibration)(&device, device_profiles.as_deref().map(PathBuf::as_path))
    {
        touch_state.set_calibration(cal);
        engine.set_calibration(cal);
        if let Ok(mut s) = state.lock() {
            s.calibration = Some(cal);
            // The effective calibration differs from the firmware
            // values exactly when a device profile overrode them.
            s.calibration_source = Some(if device.calibration() == Some(cal) {
                CalibrationSource::Device
            } else {
                CalibrationSource::Profile
            });
        }
    }
    // Report the calibration the engine actually uses (profile override
    // applied), not the firmware-reported one - clients size their field
    // sets from these millimeter bounds.
    let device_info = DeviceInfo::of(device.name(), engine.calibration().as_ref());
    if let Ok(mut s) = state.lock() {
        s.device_info = Some(device_info.clone());
    }
    runtime.set_device_info(device_info);

    let mut stable_count: Option<usize> = None;
    let mut stable_frames: u32 = 0;
    let device_fd = device.as_raw_fd();
    let session_start = std::time::Instant::now();
    let mut frame_no = 0u64;
    let mut recording: Option<RecordingSession> = None;

    loop {
        // Periodic wake for queued control commands even without input.
        let mut pfd = libc::pollfd {
            fd: device_fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut pfd, 1, 250) };
        if ready < 0 {
            if unsafe { *libc::__errno_location() } == libc::EINTR {
                continue;
            }
            error!("poll error - evdev loop exits");
            break;
        }
        if ready == 0 {
            (hooks.drain_control_commands)(
                &mut runtime,
                &mut recording,
                &mut engine,
                device.name(),
            );
            runtime.set_recording_status(recording.as_ref().map(hooks.status_of_session));
            if let Ok(mut s) = state.lock() {
                s.message = runtime.message().map(str::to_string);
                s.visible = runtime.gui_visible();
                s.rotation = runtime.display_rotation();
                s.dead_zones_visible = runtime.dead_zones_visible();
                s.spells_visible = runtime.spells_visible();
                s.dead_zones = engine.dead_zones().to_vec();
                s.trajectories = engine
                    .gesture_trajectories()
                    .values()
                    .map(|traj| traj.points.clone())
                    .collect();
                s.edge_margin = engine.edge_margin();
                s.edge_margin_visible = runtime.edge_margin_visible();
                s.fields_visible = runtime.fields_visible();
                s.field_sets = engine.field_sets().to_vec();
                s.field_set_rows = field_set_rows(&engine);
                s.profiles = handle.profiles();
                s.ruler_visible = runtime.ruler_visible();
                s.grid_visible = runtime.grid_visible();
                s.positions_visible = runtime.positions_visible();
                s.touch_passthrough = runtime.touch_passthrough();
                s.filter_mode = runtime.filter_mode();
                s.completion = engine.completion();
                s.legacy_mode = runtime.legacy_mode();
                s.filter_stability = runtime
                    .filter_stability()
                    .map(|frames| frames.as_u64())
                    .unwrap_or(0);
                s.metrics = runtime.metrics().clone();
                s.pattern_styles = engine
                    .patterns()
                    .iter()
                    .map(|pattern| crate::pattern_style::PatternStyle {
                        pattern_id: pattern.id,
                        color: pattern.color,
                        shape: pattern.shape.clone(),
                    })
                    .collect();
                s.incoming_contacts = engine.incoming_contacts();
                s.edge_filtered = engine.edge_filtered();
                let (mode_suppressed, forwarded) =
                    virtual_suppression(&engine, runtime.filter_mode(), (&*s.touches).into());
                s.mode_suppressed = mode_suppressed;
                s.forwarded = forwarded;
                s.field_set_count = engine.field_set_count();
                s.field_count = engine.field_count();
                s.occupied_fields = engine.occupied_field_count();
                if let Some(request) = s.background_alpha_request.take() {
                    runtime.set_background_alpha(request.percent().into());
                }
                s.background_alpha =
                    BackgroundAlpha::from_percent(runtime.background_alpha().percent());
            }
            runtime.publish_snapshot(engine.touches(), engine.tracked());
            continue;
        }

        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                error!(error = %e, "device read failed - evdev loop exits");
                break;
            }
        };
        for raw in events {
            if let Some(session) = &recording {
                session.record_event(&raw, session_start.elapsed().as_secs_f64());
            }
            touch_state.process(&raw);

            // The engine builds its own touch state from every event —
            // feeding it only SYN_REPORTs leaves that state empty and no
            // figure ever matches.
            let frame = if mode_is_live(&state) {
                if recording.is_some() {
                    engine.process_with_diagnostics(&raw)
                } else {
                    engine.process(&raw)
                }
            } else {
                None
            };
            if let Some(ref frame) = frame {
                frame_no += 1;
                runtime.publish(frame, engine.tracked());
                (hooks.drain_control_commands)(
                    &mut runtime,
                    &mut recording,
                    &mut engine,
                    device.name(),
                );
                runtime.set_recording_status(recording.as_ref().map(hooks.status_of_session));
                if let Ok(mut s) = state.lock() {
                    s.message = runtime.message().map(str::to_string);
                    s.visible = runtime.gui_visible();
                    s.rotation = runtime.display_rotation();
                    s.dead_zones_visible = runtime.dead_zones_visible();
                    s.spells_visible = runtime.spells_visible();
                    s.dead_zones = engine.dead_zones().to_vec();
                    s.trajectories = engine
                        .gesture_trajectories()
                        .values()
                        .map(|traj| traj.points.clone())
                        .collect();
                    if let Some(matched) = frame.spell_matches.first() {
                        s.last_spell = Some((matched.clone(), std::time::Instant::now()));
                    }
                    s.dead_zone_filtered = frame.dead_zone_filtered;
                    s.edge_margin = engine.edge_margin();
                    s.edge_margin_visible = runtime.edge_margin_visible();
                    s.fields_visible = runtime.fields_visible();
                    s.field_sets = engine.field_sets().to_vec();
                    s.field_set_rows = field_set_rows(&engine);
                    s.profiles = handle.profiles();
                    s.ruler_visible = runtime.ruler_visible();
                    s.grid_visible = runtime.grid_visible();
                    s.positions_visible = runtime.positions_visible();
                    s.touch_passthrough = runtime.touch_passthrough();
                    s.filter_mode = runtime.filter_mode();
                    s.completion = engine.completion();
                    s.legacy_mode = runtime.legacy_mode();
                    s.filter_stability = runtime
                        .filter_stability()
                        .map(|frames| frames.as_u64())
                        .unwrap_or(0);
                    s.metrics = runtime.metrics().clone();
                    s.pattern_styles = engine
                        .patterns()
                        .iter()
                        .map(|pattern| crate::pattern_style::PatternStyle {
                            pattern_id: pattern.id,
                            color: pattern.color,
                            shape: pattern.shape.clone(),
                        })
                        .collect();
                    s.incoming_contacts = engine.incoming_contacts();
                    s.edge_filtered = engine.edge_filtered();
                    let (mode_suppressed, forwarded) = virtual_suppression(
                        &engine,
                        runtime.filter_mode(),
                        (&frame.touches).into(),
                    );
                    s.mode_suppressed = mode_suppressed;
                    s.forwarded = forwarded;
                    s.field_set_count = engine.field_set_count();
                    s.field_count = engine.field_count();
                    s.occupied_fields = engine.occupied_field_count();
                    if let Some(request) = s.background_alpha_request.take() {
                        runtime.set_background_alpha(request.percent().into());
                    }
                    s.background_alpha =
                        BackgroundAlpha::from_percent(runtime.background_alpha().percent());
                }
                if let Some(profile) = runtime.training_tick(&frame.touches) {
                    info!(name = %profile.name, "training completed");
                    runtime.set_message(None);
                    (hooks.register_profile)(&mut runtime, &mut engine, profile);
                }
                if let Some(session) = &mut recording {
                    session.record_frame(
                        FrameNumber::new(frame_no),
                        Milliseconds::from(session_start.elapsed()),
                        frame,
                        engine.tracked(),
                    );
                }
            }

            if raw.is_syn_report() {
                // In live mode the frame's own touch list is authoritative:
                // `touch_indices` in matches index into it (slot-sorted).
                // The GUI's touch_state (HashMap order) would misalign them
                // and draw figure lines across objects.
                let touches = match &frame {
                    Some(engine_frame) => engine_frame.touches.clone(),
                    None => touch_state.active_touches(),
                };
                let count = touches.len();
                let matches = frame.map(|f| f.matches).unwrap_or_default();

                // Stability check (for training mode).
                if stable_count == Some(count) && count >= 3 {
                    stable_frames += 1;
                } else {
                    stable_count = Some(count);
                    stable_frames = 0;
                }

                if let Ok(mut s) = state.lock() {
                    s.touches = touches;
                    s.tracked = engine.tracked().to_vec();
                    s.matches = matches;
                    s.stable_frames = stable_frames;
                    s.stable = stable_frames >= STABLE_THRESHOLD && count >= 3;
                }
            }
        }
    }
}
