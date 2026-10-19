//! Filter-proxy mode (FILTER_PROXY_VIRTUAL.md).

use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Write;
use std::path::Path;

use miette::IntoDiagnostic;
use miette::Result;
use tangible_objects_engine::Engine;
use tangible_objects_engine::LegacyMode;
use tangible_objects_engine::Pipeline;
use tangible_objects_engine::TouchDevice;
use tangible_objects_engine::VirtualTouchscreen;
use tangible_objects_model::CompletionMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::PatternSet;
use tangible_objects_model::StageName;
use tangible_objects_model::TouchPointSlot;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::DeviceInfo;
use tangible_objects_runtime::EngineRuntime;
use tangible_objects_runtime::LifecycleEvent;
use tangible_objects_runtime::ProfileInfo;

use super::control::apply_registration;
use super::control::spawn_interfaces;
use super::detection_config::DetectionConfig;
use super::interfaces::Interfaces;
use super::setup::effective_calibration;

/// Filter-proxy mode (FILTER_PROXY_VIRTUAL.md): grabs the real device and
/// re-emits its event stream on a uinput clone, minus the slots a matched
/// tangible occupies.
///
/// Only exact matches suppress - partial candidates carry a widened
/// tolerance that would eat finger touches resembling a profile. A matched
/// slot stays suppressed across transient match loss until its tracking
/// contact ends or the filter's hysteresis expires (Figur7/8-style flappy
/// pins must not leak to the OS). `SetFilterMode` control commands switch
/// the suppression policy live (off/immediate/gated/guarded).
pub fn run_filter(
    path: &Path,
    patterns: PatternSet<'_>,
    interfaces: Interfaces,
    stability: Option<FrameCount>,
    legacy: Option<LegacyMode>,
    config: DetectionConfig,
    device_profiles: Option<&Path>,
) -> Result<()> {
    let mut device = TouchDevice::open(path).into_diagnostic()?;
    println!("Filter-Proxy: {}", path.display());
    if let Some(name) = device.name() {
        println!("Gerät: {name}");
    }

    let mut engine = Engine::new(patterns.to_vec());
    let calibration = effective_calibration(&device, device_profiles);
    if let Some(cal) = calibration {
        println!(
            "Kalibrierung: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    }
    if let Some(mm) = config.edge {
        engine.set_edge_margin(mm);
        println!("Edge-Filter: {mm} mm");
    }
    engine.set_fields(config.fields);
    engine.set_completion(config.completion);

    let mut virtual_device = VirtualTouchscreen::create(device.inner()).into_diagnostic()?;
    device.grab().into_diagnostic()?;
    println!("EVIOCGRAB aktiv - alle Events laufen über die Proxy");
    println!("uinput-Klon erstellt - Tangibles werden unterdrückt");
    println!("Ctrl+C zum Beenden\n");

    let profiles: Vec<ProfileInfo> = patterns.iter().map(ProfileInfo::from).collect();
    let (mut runtime, handle) = EngineRuntime::new(Vec::new(), profiles);
    runtime.set_completion(config.completion);
    runtime.set_field_sets(engine.field_sets().to_vec());
    spawn_interfaces(&handle, &interfaces);
    runtime.set_device_info(DeviceInfo::of(device.name(), device.calibration().as_ref()));

    let mut pipeline = Pipeline::new();
    // Edge ghosts leak to the virtual device unless suppressed - the
    // edge stage drops contacts inside the margin (the engine still
    // filters them from matching via `set_edge_margin` above).
    if let (Some(margin), Some(cal)) = (config.edge, calibration) {
        pipeline.push_edge_stage(margin, cal.surface_region());
    }
    if !config.zones.is_empty() {
        pipeline.push_zone_stage(config.zones.clone());
        println!("Zonen-Maskierung: {} Region(en)", config.zones.len());
    }
    // A stage exists per scope tagged on any pattern - lazily kept in
    // sync as registrations introduce new scope names.
    for scope in engine.stage_scopes() {
        pipeline.ensure_tangible_stage(scope.clone());
    }
    // `suppress`-flagged field sets live in the pipeline's field stage.
    pipeline.set_field_sets(engine.field_sets().to_vec());
    if let Some(gate) = stability {
        pipeline.set_mode(tangible_objects_model::FilterMode::default(), Some(gate));
        println!("Stabilitäts-Gate: {gate} Frames");
    }
    if let Some(legacy_mode) = legacy {
        pipeline.set_legacy_mode(legacy_mode);
        println!("Legacy-Events (BTN_TOUCH/ABS_X/Y): {legacy_mode}");
    }
    let mut frame: u64 = 0;
    let device_fd = device.as_raw_fd();
    loop {
        // Commands run on every loop turn - including poll timeouts - so a
        // queued SetFilterMode applies while the device is idle.
        while let Ok(command) = runtime.try_recv_command() {
            match command {
                ControlCommand::SetFilterMode { mode, stability } => {
                    pipeline.set_mode(mode, stability);
                    runtime.emit_lifecycle(LifecycleEvent::FilterModeChanged(mode));
                    println!("\nFilter-Modus: {mode}");
                }
                ControlCommand::SetCompletion(config) => {
                    engine.set_completion(config);
                    runtime.set_completion(config);
                    runtime.emit_lifecycle(LifecycleEvent::CompletionChanged(config));
                    println!(
                        "\nCompletion: mode={} radius={} max-missing={} persist={}",
                        config.mode, config.radius_factor, config.max_missing, config.persist
                    );
                }
                other => {
                    apply_registration(&mut runtime, &mut engine, other);
                    // New profiles may carry a `stage` scope not seen
                    // before - create the stage lazily; field sets flow
                    // to the pipeline's field stage.
                    for scope in engine.stage_scopes() {
                        pipeline.ensure_tangible_stage(scope.clone());
                    }
                    pipeline.set_field_sets(engine.field_sets().to_vec());
                }
            }
        }

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
            let engine_frame = engine.process(&raw);
            pipeline.push(raw);
            let Some(engine_frame) = engine_frame else {
                continue;
            };
            frame += 1;

            // Exact matches always suppress; under CompletionMode::Exact a
            // completed partial suppresses its matched *and* claimed slots.
            // Scoped by stage: a match of a `stage = "x"` pattern feeds
            // only that stage's classification (EVENT_PIPELINE.md).
            let completion_exact = engine.completion().mode == CompletionMode::Exact;
            let mut scoped: HashMap<Option<StageName>, HashSet<TouchPointSlot>> = HashMap::new();
            for matched in &engine_frame.matches {
                if !matched.is_partial()
                    || (completion_exact && !matched.claimed_indices.is_empty())
                {
                    let scope = engine
                        .pattern(matched.pattern_id)
                        .and_then(|pattern| pattern.stage.clone());
                    scoped.entry(scope).or_default().extend(
                        matched
                            .all_indices()
                            .map(|index| engine_frame.touches[index].slot),
                    );
                }
            }
            // `suppress`-enabled field sets suppress via the pipeline's
            // field stage (SINGLE_CONTACT_FIELD.md) - the daemon merge is
            // gone.
            let suppressed: usize = scoped.values().map(HashSet::len).sum();
            let out = pipeline.finish_scoped(&scoped, (&engine_frame.touches).into());
            virtual_device.emit(&out).into_diagnostic()?;

            print!(
                "\rFrame {frame} | {} Touch | {} Figur | {} unterdrückt     ",
                engine_frame.touches.len(),
                engine_frame.matches.len(),
                suppressed
            );
            std::io::stdout().flush().ok();
        }
    }
    Ok(())
}
