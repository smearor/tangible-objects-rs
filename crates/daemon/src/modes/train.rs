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
use tracing::error;
use tracing::info;
use tracing::warn;

/// Training mode: waits for a stable set of >=3 touch points (same count over
/// [`STABLE_THRESHOLD`] frames), then saves the profile on Enter.
pub fn run_train(
    path: &Path,
    name: &str,
    tolerance: impl Into<Millimeters>,
    profiles_dir: Option<&str>,
    device_profiles: Option<&Path>,
    import_slots: bool,
) -> Result<()> {
    let tolerance = tolerance.into();
    let mut device = TouchDevice::open(path).into_diagnostic()?;
    if import_slots {
        device.queue_state_import();
    }

    info!(device = %path.display(), name, "training mode");
    if let Some(dev_name) = device.name() {
        info!("device: {dev_name}");
    }
    info!("place the figure on the touch surface and hold it still.");
    info!("press Enter to capture, Ctrl+C to abort.\n");

    let mut engine = Engine::new(Vec::new());
    if let Some(cal) = effective_calibration(&device, device_profiles) {
        info!(
            "calibration: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    } else {
        warn!("no calibration readable from device - profile saved in raw coordinates");
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
            error!("poll error - training loop exits");
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
                    error!(error = %e, "device read failed - training loop exits");
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
                    println!("training \"{}\": {} touch point(s)", name, count);
                    if count >= 3 {
                        if stable_frames >= STABLE_THRESHOLD {
                            println!("STABLE - press Enter to save");
                        } else {
                            println!("stabilizing... {}/{}", stable_frames, STABLE_THRESHOLD);
                        }
                        for (i, t) in touches.iter().enumerate() {
                            println!("  point {i}: ({:.1}, {:.1}) mm", t.pos.x, t.pos.y);
                        }
                    } else {
                        println!("waiting for at least 3 touch points...");
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

        info!(path = %filepath.display(), "profile saved");
        info!("  name: {}", profile.name);
        info!("  points: {}", profile.num_points);
        info!(
            "  distances ({}): {:?}",
            profile.sorted_sides.len(),
            profile
                .sorted_sides
                .iter()
                .map(|d| d.as_f64())
                .collect::<Vec<_>>()
        );
        info!("  tolerance: {}", profile.tolerance);

        match profile.save(&filepath) {
            Ok(()) => info!("success! profile usable with --live or --replay"),
            Err(e) => error!(error = %e, "profile save failed"),
        }
    } else {
        info!("training aborted.");
    }
    Ok(())
}
