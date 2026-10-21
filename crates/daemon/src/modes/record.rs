//! Recording mode (TRAINING.md): live detection plus JSONL/evtest writers.

use std::io::Write;
use std::os::unix::io::AsRawFd;
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
use tangible_objects_model::RecordLabel;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::DeviceInfo;
use tangible_objects_runtime::EngineRuntime;
use tangible_objects_runtime::LifecycleEvent;
use tangible_objects_runtime::ProfileInfo;
use tracing::warn;

use super::control::apply_registration;
use super::control::spawn_interfaces;
use super::control::status_of_session;
use super::interfaces::Interfaces;
use super::setup::effective_calibration;
use tangible_objects_model::DetectionConfig;

/// Number of frames a keypress label applies to (~1 s at 120 Hz report rate).
const LABEL_FRAME_COUNT: u64 = 120;

/// Recording mode: like [`run_live`], additionally writing every frame to
/// `<base>.jsonl` and every raw event to `<base>.evtest` (TRAINING.md).
///
/// Matching runs through [`Engine::process_with_diagnostics`], so every
/// tested candidate lands in the frame record. Enter stops the recording
/// cleanly (drains and flushes both writers); Ctrl+C also ends it but may
/// lose the buffered tail. `SetRecordLabel`/`StopRecording` control commands
/// are honored live - API-driven labeling for application sessions.
pub fn run_record(
    path: &Path,
    base: &Path,
    label: Option<RecordLabel>,
    patterns: PatternSet<'_>,
    interfaces: Interfaces,
    config: DetectionConfig,
    device_profiles: Option<&Path>,
) -> Result<()> {
    let mut device = TouchDevice::open(path).into_diagnostic()?;

    println!("Record-Modus: {}", path.display());
    if let Some(name) = device.name() {
        println!("Gerät: {name}");
    }

    let mut engine = Engine::new(patterns.to_vec());
    if let Some(cal) = effective_calibration(&device, device_profiles) {
        println!(
            "Kalibrierung: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    } else {
        eprintln!(
            "Warnung: Keine Kalibrierung vom Gerät lesbar. Positionen werden als Rohwerte aufgezeichnet."
        );
    }
    if let Some(mm) = config.edge {
        engine.set_edge_margin(mm);
        println!("Edge-Filter: {mm} mm");
    }
    engine.set_fields(config.fields);
    engine.set_completion(config.completion);

    let mut session = RecordingSession::start(
        base.to_path_buf(),
        device.name(),
        engine.calibration().as_ref(),
        label,
    )
    .into_diagnostic()?;
    println!(
        "Aufzeichnung: {}.jsonl + {}.evtest",
        base.display(),
        base.display()
    );
    println!(
        "Enter ohne Text beendet - <label>+Enter labelt die nächsten {LABEL_FRAME_COUNT} Frames.\n"
    );

    let profiles: Vec<ProfileInfo> = patterns.iter().map(ProfileInfo::from).collect();
    let (mut runtime, handle) = EngineRuntime::new(Vec::new(), profiles);
    runtime.set_completion(config.completion);
    runtime.set_field_sets(engine.field_sets().to_vec());
    spawn_interfaces(&handle, &interfaces);
    runtime.set_device_info(DeviceInfo::of(device.name(), device.calibration().as_ref()));

    let mut frame_no: u64 = 0;
    let session_start = Instant::now();

    // Label input via poll() - wait on device fd and stdin simultaneously.
    let device_fd = device.as_raw_fd();
    let stdin_fd = std::io::stdin().as_raw_fd();
    let mut stdin_buf = [0u8; 64];
    let mut stdin_line = String::new();

    'recording: loop {
        let mut fds = [
            libc::pollfd {
                fd: device_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: stdin_fd,
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        let ret = unsafe { libc::poll(fds.as_mut_ptr(), 2, 500) };
        if ret < 0 {
            if unsafe { *libc::__errno_location() } == libc::EINTR {
                continue;
            }
            eprintln!("poll Fehler");
            break;
        }

        // Control commands run on every loop turn - including poll timeouts -
        // so `stop_recording` flushes even while the device is idle.
        while let Ok(command) = runtime.try_recv_command() {
            match command {
                ControlCommand::SetFilterMode { mode, .. } => {
                    warn!(%mode, "filter mode command - only honored in --filter mode");
                }
                ControlCommand::TrainProfile(name) => {
                    warn!(name, "remote training command - not yet wired");
                }
                ControlCommand::StartRecording { .. } => {
                    warn!("already recording - StartRecording ignored");
                }
                ControlCommand::StopRecording => {
                    let recorded = session.recorded_frames();
                    runtime.emit_lifecycle(LifecycleEvent::RecordingStopped { frames: recorded });
                    break 'recording;
                }
                ControlCommand::SetRecordLabel(new_label) => {
                    session.set_label(new_label);
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
                other => apply_registration(&mut runtime, &mut engine, other),
            }
        }

        // A stdin line marks the next LABEL_FRAME_COUNT frames; an empty line
        // or "q" ends the recording cleanly.
        if fds[1].revents & libc::POLLIN != 0 {
            let n =
                unsafe { libc::read(stdin_fd, stdin_buf.as_mut_ptr() as *mut _, stdin_buf.len()) };
            if n > 0 {
                for &byte in stdin_buf.iter().take(n as usize) {
                    match byte {
                        b'\n' | b'\r' => {
                            let text = stdin_line.trim().to_string();
                            stdin_line.clear();
                            if text.is_empty() || text == "q" {
                                break 'recording;
                            }
                            session.mark_label(RecordLabel::new(&text), LABEL_FRAME_COUNT);
                            println!(
                                "\nLabel \"{text}\" für die nächsten {LABEL_FRAME_COUNT} Frames"
                            );
                        }
                        _ => stdin_line.push(byte as char),
                    }
                }
            }
        }

        if fds[0].revents & libc::POLLIN != 0 {
            let events = match device.fetch_events() {
                Ok(events) => events,
                Err(e) => {
                    eprintln!("Lesefehler: {e}");
                    break;
                }
            };

            for raw in events {
                session.record_event(&raw, session_start.elapsed().as_secs_f64());
                let Some(engine_frame) = engine.process_with_diagnostics(&raw) else {
                    continue;
                };
                frame_no += 1;
                runtime.publish(&engine_frame, engine.tracked());

                session.record_frame(
                    FrameNumber::new(frame_no),
                    Milliseconds::from(session_start.elapsed()),
                    &engine_frame,
                    engine.tracked(),
                );
                runtime.set_recording_status(Some(status_of_session(&session)));

                let active = engine.tracked();
                print!("\x1b[2J\x1b[H");
                println!(
                    "Frame {frame_no} | {} Touch-Punkt(e) | {} Figur(en) | Label: {}",
                    engine.touch_count(),
                    active.len(),
                    session.label().map(|l| l.as_str()).unwrap_or("-"),
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

    runtime.set_recording_status(None);
    let recorded = session.recorded_frames();
    let frame_drops = session.dropped_frames();
    let raw_drops = session.dropped_events();
    session.finish().into_diagnostic()?;
    println!("\n--- Aufzeichnung beendet ---");
    println!("Frames: {frame_no} (aufgezeichnet: {recorded}, gedroppt: {frame_drops})");
    println!("Raw-Events gedroppt: {raw_drops}");
    Ok(())
}
