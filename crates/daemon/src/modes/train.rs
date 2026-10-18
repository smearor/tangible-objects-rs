//! Training mode: capture a stable contact set as a figure profile.

use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::path::PathBuf;

use miette::IntoDiagnostic;
use miette::Result;
use tangible_objects_engine::Engine;
use tangible_objects_engine::TouchDevice;
use tangible_objects_model::Millimeters;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::Vec2;

use super::setup::effective_calibration;

/// Training mode: waits for a stable set of >=3 touch points (same count over
/// [`STABLE_THRESHOLD`] frames), then saves the profile on Enter.
pub fn run_train(
    path: &Path,
    name: &str,
    tolerance: impl Into<Millimeters>,
    profiles_dir: Option<&str>,
    device_profiles: Option<&Path>,
) -> Result<()> {
    let tolerance = tolerance.into();
    let mut device = TouchDevice::open(path).into_diagnostic()?;

    println!("Trainingsmodus: {} - Figur \"{}\"", path.display(), name);
    if let Some(dev_name) = device.name() {
        println!("Gerät: {dev_name}");
    }
    println!("Lege die Figur auf die Touchfläche und halte sie still.");
    println!("Drücke Enter zum Erfassen, Ctrl+C zum Abbrechen.\n");

    let mut engine = Engine::new(Vec::new());
    if let Some(cal) = effective_calibration(&device, device_profiles) {
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

    // Enter detection via poll() - wait on device fd and stdin simultaneously.
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
                            println!("STABIL - Drücke Enter zum Speichern");
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
