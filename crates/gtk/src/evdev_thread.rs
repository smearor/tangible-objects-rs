//! The device loop feeding [`crate::SharedState`] - mirrors the
//! headless modes' engine loop, plus the draw-side writes.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use tangible_objects_engine::Engine;
use tangible_objects_engine::RecordingSession;
use tangible_objects_engine::TouchDevice;
use tangible_objects_engine::TouchState;
use tangible_objects_model::DetectionConfig;
use tangible_objects_model::FrameNumber;
use tangible_objects_model::Milliseconds;
use tangible_objects_model::PatternBuf;
use tangible_objects_model::ProfileInfo;
use tangible_objects_runtime::DeviceInfo;
use tangible_objects_runtime::EngineRuntime;

use crate::GuiHooks;
use crate::shared_state::SharedState;
use crate::shared_state::mode_is_live;

/// Consecutive same-size frames before a point set counts as stable
/// for profile capture.
pub(crate) const STABLE_THRESHOLD: u32 = 10;

/// Device loop thread: opens the evdev device, runs the engine per
/// `SYN_REPORT` frame, publishes snapshots/events through the runtime
/// and writes the GUI's [`SharedState`]. Daemon semantics (interfaces,
/// control commands, recording) arrive via [`GuiHooks`].
pub fn evdev_thread(
    device_path: String,
    patterns: PatternBuf,
    state: Arc<Mutex<SharedState>>,
    config: DetectionConfig,
    device_profiles: Option<Arc<PathBuf>>,
    hooks: GuiHooks,
) {
    let mut device = match TouchDevice::open(std::path::Path::new(&device_path)) {
        Ok(d) => d,
        Err(e) => {
            if let Ok(mut s) = state.lock() {
                s.device_error = Some(format!("Gerät konnte nicht geöffnet werden: {e}"));
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
    engine.set_fields(config.fields);
    engine.set_completion(config.completion);

    // Event hub for interface crates (GraphQL/D-Bus/MCP) - the GUI runs
    // its own device loop, so it owns the runtime side like the
    // headless modes do.
    let profiles: Vec<ProfileInfo> = engine.patterns().iter().map(ProfileInfo::from).collect();
    let (mut runtime, handle) = EngineRuntime::new(Vec::new(), profiles);
    runtime.set_completion(config.completion);
    runtime.set_field_sets(engine.field_sets().to_vec());
    (hooks.spawn_interfaces)(&handle);
    runtime.set_device_info(DeviceInfo::of(device.name(), device.calibration().as_ref()));

    let mut touch_state = TouchState::new();
    if let Some(cal) =
        (hooks.effective_calibration)(&device, device_profiles.as_deref().map(PathBuf::as_path))
    {
        touch_state.set_calibration(cal);
        engine.set_calibration(cal);
        if let Ok(mut s) = state.lock() {
            s.calibration = Some(cal);
        }
    }

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
            eprintln!("poll Fehler");
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
            }
            runtime.publish_snapshot(engine.touches(), engine.tracked());
            continue;
        }

        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                eprintln!("Lesefehler: {e}");
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
                }
                if let Some(profile) = runtime.training_tick(&frame.touches) {
                    println!("\nTraining abgeschlossen: {}", profile.name);
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
