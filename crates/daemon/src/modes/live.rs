//! Live detection loop on a real evdev device.

use std::io::Write;
use std::path::Path;
use std::time::Instant;

use miette::IntoDiagnostic;
use miette::Result;
use tangible_objects_engine::Engine;
use tangible_objects_engine::RecordingSession;
use tangible_objects_engine::TouchDevice;
use tangible_objects_model::FrameNumber;
use tangible_objects_model::Milliseconds;
use tangible_objects_model::PatternSet;
use tangible_objects_runtime::DeviceInfo;
use tangible_objects_runtime::EngineRuntime;
use tangible_objects_runtime::ProfileInfo;

use super::control::drain_control_commands;
use super::control::spawn_interfaces;
use super::control::status_of_session;
use super::control::tick_training;
use super::interfaces::Interfaces;
use super::setup::effective_calibration;
use tangible_objects_model::DetectionConfig;
use tracing::error;
use tracing::info;
use tracing::warn;

/// Live detection loop on a real evdev device.
///
/// `interfaces` selects which Phase-2 interfaces are served on dedicated
/// threads (GraphQL HTTP/WebSocket, D-Bus session bus).
pub fn run_live(
    path: &Path,
    patterns: PatternSet<'_>,
    interfaces: Interfaces,
    config: DetectionConfig,
    device_profiles: Option<&Path>,
) -> Result<()> {
    let mut device = TouchDevice::open(path).into_diagnostic()?;
    if config.import_slots {
        device.queue_state_import();
    }

    info!(device = %path.display(), "live mode");
    if let Some(name) = device.name() {
        info!("device: {name}");
    }

    let mut engine = Engine::new(patterns.to_vec());
    if let Some(cal) = effective_calibration(&device, device_profiles) {
        info!(
            "calibration: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    } else {
        warn!("no calibration readable from device - profiles must be in raw coordinates");
    }
    if let Some(mm) = config.edge {
        engine.set_edge_margin(mm);
        info!("edge filter: {mm} mm");
    }
    engine.set_fields(config.fields);
    engine.set_dead_zones(config.dead_zones);
    engine.set_completion(config.completion);
    info!("Ctrl+C to quit\n");

    // Event hub for interface crates (GraphQL/D-Bus/MCP, Phase 2).
    // No set definitions are loaded yet - the set tracker idles.
    let profiles: Vec<ProfileInfo> = patterns.iter().map(ProfileInfo::from).collect();
    let (mut runtime, handle) = EngineRuntime::new(Vec::new(), profiles);
    runtime.set_completion(config.completion);
    runtime.set_field_sets(engine.field_sets().to_vec());
    runtime.set_dead_zones(engine.dead_zones().to_vec());
    spawn_interfaces(&handle, &interfaces);
    // Report the calibration the engine actually uses (profile override
    // applied), not the firmware-reported one - clients size their field
    // sets from these millimeter bounds.
    runtime.set_device_info(DeviceInfo::of(device.name(), engine.calibration().as_ref()));

    let mut frame: u64 = 0;
    let session_start = Instant::now();
    let mut recording: Option<RecordingSession> = None;
    let device_fd = device.as_raw_fd();

    loop {
        // Wake periodically even without device events: queued control
        // commands (start/stop recording) must not wait for input - a
        // StopRecording on an idle screen has to flush immediately.
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
            error!("poll error - event loop exits");
            break;
        }
        if ready == 0 {
            drain_control_commands(&mut runtime, &mut recording, &mut engine, device.name());
            runtime.set_recording_status(recording.as_ref().map(status_of_session));
            runtime.publish_snapshot(engine.touches(), engine.tracked());
            continue;
        }

        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                error!(error = %e, "device read failed - event loop exits");
                break;
            }
        };

        for raw in events {
            if let Some(session) = &recording {
                session.record_event(&raw, session_start.elapsed().as_secs_f64());
            }
            let engine_frame = if recording.is_some() {
                engine.process_with_diagnostics(&raw)
            } else {
                engine.process(&raw)
            };
            if let Some(engine_frame) = engine_frame {
                frame += 1;
                runtime.publish(&engine_frame, engine.tracked());
                drain_control_commands(&mut runtime, &mut recording, &mut engine, device.name());
                tick_training(&mut runtime, &mut engine, &engine_frame.touches);

                if let Some(session) = &mut recording {
                    session.record_frame(
                        FrameNumber::new(frame),
                        Milliseconds::from(session_start.elapsed()),
                        &engine_frame,
                        engine.tracked(),
                    );
                }
                runtime.set_recording_status(recording.as_ref().map(status_of_session));

                let active = engine.tracked();
                print!("\x1b[2J\x1b[H");
                println!(
                    "Frame {frame} | {} touch point(s) | {} figure(s) | Rec: {}",
                    engine.touch_count(),
                    active.len(),
                    recording
                        .as_ref()
                        .and_then(|s| s.label().map(|l| l.as_str().to_string()))
                        .unwrap_or_else(|| if recording.is_some() {
                            "on".to_string()
                        } else {
                            "off".to_string()
                        }),
                );
                for t in active {
                    let deg = t.rotation_rad.to_degrees();
                    println!(
                        "  {} (id={}): center=({:.0}, {:.0}) rotation={:.1}°",
                        t.pattern_name, t.pattern_id, t.center.x, t.center.y, deg
                    );
                }
                std::io::stdout().flush().ok();
            }
        }
    }
    Ok(())
}
