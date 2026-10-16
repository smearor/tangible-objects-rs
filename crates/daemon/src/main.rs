//! Command-line entry point of the `tangible-objects` daemon binary.
//!
//! Modes: `--list` (enumerate devices), `--replay` (replay an evtest
//! recording), `--live` (live detection), `--record` (frame + raw recording),
//! `--train` (record a figure profile), `--gui` (GTK visualization,
//! feature-gated).

mod modes;

#[cfg(feature = "gui")]
mod gtk_app;

use std::net::SocketAddr;
use std::path::Path;

use miette::IntoDiagnostic;
use miette::Result;
use tangible_objects_model::Millimeters;
use tangible_objects_model::RecordLabel;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut mode: Option<&str> = None;
    let mut device: Option<String> = None;
    let mut file: Option<String> = None;
    let mut profiles_dir: Option<String> = None;
    let mut name: Option<String> = None;
    let mut tolerance: Option<f64> = None;
    let mut graphql_listen: Option<String> = None;
    let mut mcp_listen: Option<String> = None;
    let mut record_base: Option<String> = None;
    let mut record_label: Option<String> = None;
    let mut dbus = false;

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
            "--record" => {
                record_base = args.get(i + 1).cloned();
                i += 1;
            }
            "--analyze" => {
                mode = Some("--analyze");
                file = args.get(i + 1).cloned();
                i += 1;
            }
            "--export-dataset" => {
                mode = Some("--export-dataset");
                file = args.get(i + 1).cloned();
                i += 1;
            }
            "--record-label" => {
                record_label = args.get(i + 1).cloned();
                i += 1;
            }
            "--graphql-listen" => {
                graphql_listen = args.get(i + 1).cloned();
                i += 1;
            }
            "--mcp-listen" => {
                mcp_listen = args.get(i + 1).cloned();
                i += 1;
            }
            "--dbus" => {
                dbus = true;
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

    // --record is a mode on its own (--record <base> --device …) but also a
    // modifier on --replay and --live.
    let effective_mode = match (mode, record_base.is_some()) {
        (Some(mode), _) => mode,
        (None, true) => "--record",
        (None, false) => "--help",
    };

    match effective_mode {
        "--replay" => {
            let path = file.ok_or_else(|| {
                miette::miette!("Usage: tangible-objects --replay --file <evtest-file>")
            })?;
            let patterns = modes::load_patterns(profiles_dir.as_deref());
            modes::run_replay(
                Path::new(&path),
                &patterns,
                record_base.as_deref().map(Path::new),
                record_label.map(RecordLabel::new),
            )
        }
        "--live" | "--record" => {
            let path = device.ok_or_else(|| {
                miette::miette!(
                    "Usage: tangible-objects --live|--record <base> --device /dev/input/eventX"
                )
            })?;
            let patterns = modes::load_patterns(profiles_dir.as_deref());
            let interfaces = parse_interfaces(graphql_listen, mcp_listen, dbus)?;
            match record_base {
                Some(base) => modes::run_record(
                    Path::new(&path),
                    Path::new(&base),
                    record_label.map(RecordLabel::new),
                    &patterns,
                    interfaces,
                ),
                None => modes::run_live(Path::new(&path), &patterns, interfaces),
            }
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
            let tol = Millimeters::new(tolerance.unwrap_or(30.0));
            modes::run_train(Path::new(&dev), &fig_name, tol, profiles_dir.as_deref())
        }
        "--analyze" => {
            let path = file.ok_or_else(|| {
                miette::miette!("Usage: tangible-objects --analyze <recording.jsonl>")
            })?;
            modes::run_analyze(Path::new(&path))
        }
        "--export-dataset" => {
            let path = file.ok_or_else(|| {
                miette::miette!("Usage: tangible-objects --export-dataset <recording.jsonl>")
            })?;
            modes::run_export_dataset(Path::new(&path))
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

/// Parses the optional interface flags into [`modes::Interfaces`].
fn parse_interfaces(
    graphql_listen: Option<String>,
    mcp_listen: Option<String>,
    dbus: bool,
) -> Result<modes::Interfaces> {
    let graphql_addr = graphql_listen
        .map(|a| a.parse::<SocketAddr>())
        .transpose()
        .into_diagnostic()?;
    let mcp_addr = mcp_listen
        .map(|a| a.parse::<SocketAddr>())
        .transpose()
        .into_diagnostic()?;
    Ok(modes::Interfaces {
        graphql_addr,
        dbus,
        mcp_addr,
    })
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
        "  tangible-objects --record <base> --device /dev/input/eventX      Aufzeichnung (<base>.jsonl + <base>.evtest)"
    );
    eprintln!(
        "  tangible-objects --replay --file <f> --record <base>             Replay deterministisch aufzeichnen"
    );
    eprintln!(
        "  tangible-objects --analyze <recording.jsonl>                     Aufzeichnung analysieren (Phase 2+3)"
    );
    eprintln!(
        "  tangible-objects --export-dataset <recording.jsonl>              Gelabeltes Dataset exportieren (Phase 3)"
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
    eprintln!("  --tolerance <mm>   Toleranz für Musterabgleich (Standard: 30.0)");
    eprintln!("  --name <name>      Figurenname (für --train)");
    eprintln!("  --record-label <label>  Ground-Truth-Label für --record");
    eprintln!("  --graphql-listen <addr>  GraphQL-Endpunkt im --live Modus (z.B. 127.0.0.1:8080)");
    eprintln!("  --mcp-listen <addr>      MCP-HTTP-Endpunkt im --live Modus (z.B. 127.0.0.1:8090)");
    eprintln!("  --dbus               D-Bus-Interface im --live Modus (Session-Bus)");
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
