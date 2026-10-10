//! Daemon modes: replay, live detection, training and device listing.

use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::path::PathBuf;

use miette::IntoDiagnostic;
use miette::Result;
use tangible_objects_engine::Engine;
use tangible_objects_engine::EngineEvent;
use tangible_objects_engine::EvtestReplay;
use tangible_objects_engine::TouchDevice;
use tangible_objects_model::Millimeters;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::TangiblePattern;
use tangible_objects_model::Vec2;

/// Loads figure patterns from a profile directory (default: `profiles/`).
pub fn load_patterns(profiles_dir: Option<&str>) -> Vec<TangiblePattern> {
    let dir = profiles_dir.unwrap_or("profiles");
    let dir_result = ProfileFile::load_dir(Path::new(dir));
    for (_, profile) in &dir_result.profiles {
        println!(
            "Profil geladen: {} ({} Punkte, {} Abstände)",
            profile.name,
            profile.num_points,
            profile.sorted_sides.len()
        );
    }
    for (path, error) in &dir_result.errors {
        eprintln!(
            "Warnung: Profil {} konnte nicht geladen werden: {}",
            path.display(),
            error
        );
    }
    let patterns = dir_result.into_patterns();
    if patterns.is_empty() {
        eprintln!(
            "Warnung: Keine Profile in {} gefunden. Erstelle ein Profil mit --train.",
            dir
        );
    }
    patterns
}

/// Replays an `evtest` recording through the detection pipeline.
pub fn run_replay(path: &Path, patterns: &[TangiblePattern]) -> Result<()> {
    let mut replay = EvtestReplay::open(path).into_diagnostic()?;
    let mut engine = Engine::new(patterns.to_vec());

    if let Some(cal) = replay.calibration() {
        println!(
            "Kalibrierung: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    } else {
        eprintln!(
            "Warnung: Keine Kalibrierung im evtest-Header gefunden. Profile müssen in Rohkoordinaten vorliegen."
        );
    }

    let mut frame_no: u64 = 0;
    let mut detected_frames: u64 = 0;

    while let Some(event) = replay.next_event() {
        let Some(frame) = engine.process(&event) else {
            continue;
        };
        let added = frame
            .events
            .iter()
            .any(|e| matches!(e, EngineEvent::TangibleAdded(_)));
        for engine_event in &frame.events {
            match engine_event {
                EngineEvent::TangibleAdded(_) => println!(
                    "[Frame {frame_no}] Figur erkannt ({} aktiv)",
                    engine.tracked().len()
                ),
                EngineEvent::TangibleRemoved(_) if engine.tracked().is_empty() => {
                    println!("[Frame {frame_no}] Figur verloren");
                }
                _ => {}
            }
        }

        let active = engine.tracked();
        if !active.is_empty() {
            detected_frames += 1;
            if detected_frames.is_multiple_of(50) || added {
                for t in active {
                    let deg = t.rotation_rad.to_degrees();
                    println!(
                        "  {} (id={}): center=({:.0}, {:.0}) rotation={:.1}°",
                        t.pattern_name, t.pattern_id, t.center.x, t.center.y, deg
                    );
                }
            }
        }
        frame_no += 1;
    }

    println!("\n--- Zusammenfassung ---");
    println!("Frames gesamt: {frame_no}");
    println!("Frames mit Erkennung: {detected_frames}");
    Ok(())
}

/// Live detection loop on a real evdev device.
pub fn run_live(path: &Path, patterns: &[TangiblePattern]) -> Result<()> {
    let mut device = TouchDevice::open(path).into_diagnostic()?;

    println!("Live-Modus: {}", path.display());
    if let Some(name) = device.name() {
        println!("Gerät: {name}");
    }

    let mut engine = Engine::new(patterns.to_vec());
    if let Some(cal) = device.calibration() {
        println!(
            "Kalibrierung: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    } else {
        eprintln!(
            "Warnung: Keine Kalibrierung vom Gerät lesbar. Profile müssen in Rohkoordinaten vorliegen."
        );
    }
    println!("Ctrl+C zum Beenden\n");

    let mut frame: u64 = 0;

    loop {
        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                eprintln!("Lesefehler: {e}");
                break;
            }
        };

        for raw in events {
            if engine.process(&raw).is_some() {
                let active = engine.tracked();
                print!("\x1b[2J\x1b[H");
                println!(
                    "Frame {frame} | {} Touch-Punkt(e) | {} Figur(en)",
                    engine.touch_count(),
                    active.len()
                );
                for t in active {
                    let deg = t.rotation_rad.to_degrees();
                    println!(
                        "  {} (id={}): center=({:.0}, {:.0}) rotation={:.1}°",
                        t.pattern_name, t.pattern_id, t.center.x, t.center.y, deg
                    );
                }
                std::io::stdout().flush().ok();
                frame += 1;
            }
        }
    }
    Ok(())
}

/// Training mode: waits for a stable set of >=3 touch points (same count over
/// [`STABLE_THRESHOLD`] frames), then saves the profile on Enter.
pub fn run_train(
    path: &Path,
    name: &str,
    tolerance: impl Into<Millimeters>,
    profiles_dir: Option<&str>,
) -> Result<()> {
    let tolerance = tolerance.into();
    let mut device = TouchDevice::open(path).into_diagnostic()?;

    println!("Trainingsmodus: {} — Figur \"{}\"", path.display(), name);
    if let Some(dev_name) = device.name() {
        println!("Gerät: {dev_name}");
    }
    println!("Lege die Figur auf die Touchfläche und halte sie still.");
    println!("Drücke Enter zum Erfassen, Ctrl+C zum Abbrechen.\n");

    let mut engine = Engine::new(Vec::new());
    if let Some(cal) = device.calibration() {
        println!(
            "Kalibrierung: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    } else {
        eprintln!(
            "Warnung: Keine Kalibrierung vom Gerät lesbar. Profil wird in Rohkoordinaten gespeichert."
        );
    }

    const STABLE_THRESHOLD: u32 = 10;
    let mut stable_count: Option<usize> = None;
    let mut stable_frames: u32 = 0;

    // Enter detection via poll() — wait on device fd and stdin simultaneously.
    let device_fd = device.as_raw_fd();
    let stdin_fd = std::io::stdin().as_raw_fd();

    let mut captured: Option<Vec<Vec2<Millimeters>>> = None;
    let mut stdin_buf = [0u8; 64];
    let mut enter_pressed = false;

    loop {
        // Poll both fds (device + stdin), 500 ms timeout.
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

        // Check stdin.
        if fds[1].revents & libc::POLLIN != 0 {
            let n =
                unsafe { libc::read(stdin_fd, stdin_buf.as_mut_ptr() as *mut _, stdin_buf.len()) };
            if n > 0 {
                for &byte in stdin_buf.iter().take(n as usize) {
                    if byte == b'\n' || byte == b'\r' {
                        enter_pressed = true;
                    }
                }
            }
        }

        // Process device events (non-blocking).
        if fds[0].revents & libc::POLLIN != 0 {
            let events = match device.fetch_events() {
                Ok(events) => events,
                Err(e) => {
                    eprintln!("Lesefehler: {e}");
                    break;
                }
            };

            for raw in events {
                if engine.process(&raw).is_some() {
                    let touches = engine.touches();
                    let count = touches.len();

                    // Stability check.
                    if stable_count == Some(count) && count >= 3 {
                        stable_frames += 1;
                    } else {
                        stable_count = Some(count);
                        stable_frames = 0;
                    }

                    print!("\x1b[2J\x1b[H");
                    println!("Trainingsmodus: \"{}\" | {} Touch-Punkt(e)", name, count);
                    if count >= 3 {
                        if stable_frames >= STABLE_THRESHOLD {
                            println!("STABIL — Drücke Enter zum Speichern");
                        } else {
                            println!("Stabilisiere... {}/{}", stable_frames, STABLE_THRESHOLD);
                        }
                        for (i, t) in touches.iter().enumerate() {
                            println!("  Punkt {i}: ({:.1}, {:.1}) mm", t.pos.x, t.pos.y);
                        }
                    } else {
                        println!("Warte auf mindestens 3 Touch-Punkte...");
                    }
                    std::io::stdout().flush().ok();

                    if enter_pressed && count >= 3 && stable_frames >= STABLE_THRESHOLD {
                        captured = Some(touches.iter().map(|t| t.pos).collect());
                        break;
                    }
                }
            }
        }

        // Check Enter even without a device event (points may already be stable).
        if enter_pressed && captured.is_none() {
            let touches = engine.touches();
            let count = touches.len();
            if count >= 3 && stable_frames >= STABLE_THRESHOLD {
                captured = Some(touches.iter().map(|t| t.pos).collect());
            }
        }

        if captured.is_some() {
            break;
        }
    }

    if let Some(points) = captured {
        let profile = ProfileFile::from_points(name, &points, tolerance);

        let dir = profiles_dir.unwrap_or("profiles");
        std::fs::create_dir_all(dir).ok();
        let filename = format!("{}/{}.toml", dir, name.to_lowercase().replace(' ', "-"));
        let filepath = PathBuf::from(&filename);

        println!("\nProfil gespeichert: {}", filepath.display());
        println!("  Name: {}", profile.name);
        println!("  Punkte: {}", profile.num_points);
        println!(
            "  Abstände ({}): {:?}",
            profile.sorted_sides.len(),
            profile
                .sorted_sides
                .iter()
                .map(|d| d.as_f64())
                .collect::<Vec<_>>()
        );
        println!("  Toleranz: {}", profile.tolerance);

        match profile.save(&filepath) {
            Ok(()) => {
                println!("\nErfolgreich! Profil kann mit --live oder --replay verwendet werden.")
            }
            Err(e) => eprintln!("Fehler beim Speichern: {e}"),
        }
    } else {
        eprintln!("\nTraining abgebrochen.");
    }
    Ok(())
}

/// Lists all `/dev/input/eventX` devices with names and multitouch markers.
pub fn list_devices() {
    let entries = TouchDevice::enumerate();
    if entries.is_empty() {
        eprintln!("Keine Eingabegeräte in /dev/input/ gefunden.");
        return;
    }
    for entry in entries {
        println!("{entry}");
    }
}
