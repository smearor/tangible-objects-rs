//! Command-line entry point of the `tangible-objects` daemon binary.
//!
//! Modes: `--list` (enumerate devices), `--replay` (replay an evtest
//! recording), `--live` (live detection), `--train` (record a figure profile),
//! `--gui` (GTK visualization, feature-gated).

mod modes;

#[cfg(feature = "gui")]
mod gtk_app;

use std::path::Path;

use miette::Result;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut mode: Option<&str> = None;
    let mut device: Option<String> = None;
    let mut file: Option<String> = None;
    let mut profiles_dir: Option<String> = None;
    let mut name: Option<String> = None;
    let mut tolerance: Option<f64> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--replay" | "--live" | "--list" | "--train" | "--gui" => {
                mode = Some(match args[i].as_str() {
                    "--replay" => "--replay",
                    "--live" => "--live",
                    "--train" => "--train",
                    "--gui" => "--gui",
                    _ => "--list",
                });
            }
            "--device" => {
                device = args.get(i + 1).cloned();
                i += 1;
            }
            "--file" => {
                file = args.get(i + 1).cloned();
                i += 1;
            }
            "--profiles" => {
                profiles_dir = args.get(i + 1).cloned();
                i += 1;
            }
            "--name" => {
                name = args.get(i + 1).cloned();
                i += 1;
            }
            "--tolerance" => {
                tolerance = args.get(i + 1).and_then(|s| s.parse().ok());
                i += 1;
            }
            "--help" | "-h" => {
                mode = Some("--help");
            }
            _ => {
                eprintln!("Unbekanntes Argument: {}", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    match mode.unwrap_or("--help") {
        "--replay" => {
            let path = file.ok_or_else(|| {
                miette::miette!("Usage: tangible-objects --replay --file <evtest-file>")
            })?;
            let patterns = modes::load_patterns(profiles_dir.as_deref());
            modes::run_replay(Path::new(&path), &patterns)
        }
        "--live" => {
            let path = device.ok_or_else(|| {
                miette::miette!("Usage: tangible-objects --live --device /dev/input/eventX")
            })?;
            let patterns = modes::load_patterns(profiles_dir.as_deref());
            modes::run_live(Path::new(&path), &patterns)
        }
        "--train" => {
            let dev = device.ok_or_else(|| {
                miette::miette!(
                    "Usage: tangible-objects --train --device /dev/input/eventX --name <name>"
                )
            })?;
            let fig_name = name.ok_or_else(|| {
                miette::miette!("Usage: tangible-objects --train --device ... --name <name>")
            })?;
            let tol = tolerance.unwrap_or(30.0);
            modes::run_train(Path::new(&dev), &fig_name, tol, profiles_dir.as_deref())
        }
        "--list" => {
            modes::list_devices();
            Ok(())
        }
        "--gui" => {
            let dev = device.ok_or_else(|| {
                miette::miette!("Usage: tangible-objects --gui --device /dev/input/eventX")
            })?;
            run_gui_mode(&dev, profiles_dir.as_deref())
        }
        _ => {
            print_help();
            Ok(())
        }
    }
}

fn print_help() {
    eprintln!("tangible-objects — kapazitive Touchfiguren-Prototyp");
    eprintln!();
    eprintln!("Usage:");
    eprintln!(
        "  tangible-objects --list                                          Eingabegeräte auflisten"
    );
    eprintln!(
        "  tangible-objects --replay --file <evtest-output.txt>             Replay einer evtest-Aufzeichnung"
    );
    eprintln!(
        "  tangible-objects --live --device /dev/input/eventX               Live von einem Eingabegerät"
    );
    eprintln!(
        "  tangible-objects --train --device /dev/input/eventX --name <name>  Profil aufnehmen und speichern"
    );
    eprintln!(
        "  tangible-objects --gui --device /dev/input/eventX                  GTK Fullscreen-Visualisierung"
    );
    eprintln!();
    eprintln!("Optionen:");
    eprintln!("  --profiles <dir>   Profil-Verzeichnis (Standard: profiles/)");
    eprintln!("  --tolerance <px>   Toleranz für Musterabgleich (Standard: 30.0)");
    eprintln!("  --name <name>      Figurenname (für --train)");
    eprintln!();
    eprintln!("Beispiel:");
    eprintln!("  tangible-objects --replay --file research/figure-tracking-4.txt");
    eprintln!("  tangible-objects --live --device /dev/input/event2");
    eprintln!("  tangible-objects --train --device /dev/input/event2 --name Ritter");
}

/// Runs the GTK visualization (feature `gui`).
#[cfg(feature = "gui")]
fn run_gui_mode(dev: &str, profiles_dir: Option<&str>) -> Result<()> {
    gtk_app::run_gui(dev, profiles_dir);
    Ok(())
}

/// Fallback when the binary was built without the `gui` feature.
#[cfg(not(feature = "gui"))]
fn run_gui_mode(_dev: &str, _profiles_dir: Option<&str>) -> Result<()> {
    eprintln!("GUI-Modus nicht kompiliert. Baue mit: cargo build --features gui");
    std::process::exit(1);
}
