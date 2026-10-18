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
use std::path::PathBuf;

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
    let mut device_profiles: Option<String> = None;
    let mut convert_input: Option<String> = None;
    let mut convert_to: Option<String> = None;
    let mut edge: Option<Millimeters> = None;
    let mut similarity_json: Option<String> = None;
    let mut stability: Option<u64> = None;
    let mut legacy: Option<String> = None;
    let mut fields_path: Option<String> = None;
    let mut affinity_factor: Option<f64> = None;
    let mut smoothing_alpha: Option<f64> = None;
    let mut completion_mode: Option<String> = None;
    let mut completion_radius: Option<f64> = None;
    let mut completion_missing: Option<u32> = None;
    let mut completion_no_persist = false;
    let mut dbus = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--replay" | "--live" | "--list" | "--train" | "--gui" | "--filter" => {
                mode = Some(match args[i].as_str() {
                    "--replay" => "--replay",
                    "--live" => "--live",
                    "--train" => "--train",
                    "--gui" => "--gui",
                    "--filter" => "--filter",
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
            "--similarity" => {
                mode = Some("--similarity");
                file = args.get(i + 1).cloned();
                i += 1;
            }
            "--similarity-json" => {
                similarity_json = args.get(i + 1).cloned();
                i += 1;
            }
            "--export-dataset" => {
                mode = Some("--export-dataset");
                file = args.get(i + 1).cloned();
                i += 1;
            }
            "--convert-device" => {
                mode = Some("--convert-device");
                convert_input = args.get(i + 1).cloned();
                i += 1;
            }
            "--to" => {
                convert_to = args.get(i + 1).cloned();
                i += 1;
            }
            "--device-profiles" => {
                device_profiles = args.get(i + 1).cloned();
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
            "--legacy" => {
                legacy = args.get(i + 1).cloned();
                i += 1;
            }
            "--stability-frames" => {
                stability = args.get(i + 1).and_then(|s| s.parse().ok());
                i += 1;
            }
            "--edge-filter-mm" => {
                edge = args
                    .get(i + 1)
                    .and_then(|s| s.parse().ok())
                    .map(Millimeters::new);
                i += 1;
            }
            "--fields" => {
                fields_path = args.get(i + 1).cloned();
                i += 1;
            }
            "--completion-mode" => {
                completion_mode = args.get(i + 1).cloned();
                i += 1;
            }
            "--completion-radius" => {
                completion_radius = args.get(i + 1).and_then(|s| s.parse().ok());
                i += 1;
            }
            "--completion-max-missing" => {
                completion_missing = args.get(i + 1).and_then(|s| s.parse().ok());
                i += 1;
            }
            "--completion-no-persist" => {
                completion_no_persist = true;
            }
            "--affinity-factor" => {
                affinity_factor = args.get(i + 1).and_then(|s| s.parse().ok());
                i += 1;
            }
            "--smoothing-alpha" => {
                smoothing_alpha = args.get(i + 1).and_then(|s| s.parse().ok());
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
                (&patterns).into(),
                record_base.as_deref().map(Path::new),
                record_label.map(RecordLabel::new),
            )
        }
        "--live" | "--record" => {
            // Without --device, auto-detect a known touchscreen by
            // vendor:product (builtin + --device-profiles registry).
            let path = modes::resolve_device(device, device_profiles.as_deref().map(Path::new))?;
            let patterns = modes::load_patterns(profiles_dir.as_deref());
            let interfaces = parse_interfaces(graphql_listen, mcp_listen, dbus)?;
            let config = modes::DetectionConfig {
                edge,
                fields: modes::load_field_sets(fields_path.as_deref()),
                completion: parse_completion(
                    completion_mode,
                    completion_radius,
                    completion_missing,
                    completion_no_persist,
                    affinity_factor,
                    smoothing_alpha,
                )?,
            };
            match record_base {
                Some(base) => modes::run_record(
                    &path,
                    Path::new(&base),
                    record_label.map(RecordLabel::new),
                    (&patterns).into(),
                    interfaces,
                    config,
                    device_profiles.as_deref().map(Path::new),
                ),
                None => modes::run_live(
                    &path,
                    (&patterns).into(),
                    interfaces,
                    config,
                    device_profiles.as_deref().map(Path::new),
                ),
            }
        }
        "--filter" => {
            // Filter proxy: grab the real device and re-emit non-tangible
            // events on a uinput clone (FILTER_PROXY_VIRTUAL.md).
            let dev = modes::resolve_device(device, device_profiles.as_deref().map(Path::new))?;
            let patterns = modes::load_patterns(profiles_dir.as_deref());
            let interfaces = parse_interfaces(graphql_listen, mcp_listen, dbus)?;
            let legacy_mode = legacy
                .as_deref()
                .map(|value| value.parse::<tangible_objects_engine::LegacyMode>())
                .transpose()
                .map_err(|_| miette::miette!("--legacy must be one of: auto, always, never"))?;
            let completion = parse_completion(
                completion_mode,
                completion_radius,
                completion_missing,
                completion_no_persist,
                affinity_factor,
                smoothing_alpha,
            )?;
            modes::run_filter(
                &dev,
                (&patterns).into(),
                interfaces,
                stability.map(tangible_objects_model::FrameCount::new),
                legacy_mode,
                modes::DetectionConfig {
                    edge,
                    fields: modes::load_field_sets(fields_path.as_deref()),
                    completion,
                },
                device_profiles.as_deref().map(Path::new),
            )
        }
        "--train" => {
            let dev = modes::resolve_device(device, device_profiles.as_deref().map(Path::new))?;
            let fig_name = name
                .ok_or_else(|| miette::miette!("Usage: tangible-objects --train --name <name>"))?;
            let tol = Millimeters::new(tolerance.unwrap_or(30.0));
            modes::run_train(
                &dev,
                &fig_name,
                tol,
                profiles_dir.as_deref(),
                device_profiles.as_deref().map(Path::new),
            )
        }
        "--analyze" => {
            let path = file.ok_or_else(|| {
                miette::miette!("Usage: tangible-objects --analyze <recording.jsonl>")
            })?;
            modes::run_analyze(Path::new(&path), profiles_dir.as_deref())
        }
        "--similarity" => {
            let dir = file.ok_or_else(|| {
                miette::miette!("Usage: tangible-objects --similarity <profiles-dir>")
            })?;
            modes::run_similarity(Path::new(&dir), similarity_json.as_deref().map(Path::new))
        }
        "--export-dataset" => {
            let path = file.ok_or_else(|| {
                miette::miette!("Usage: tangible-objects --export-dataset <recording.jsonl>")
            })?;
            modes::run_export_dataset(Path::new(&path))
        }
        "--convert-device" => {
            let input = convert_input.ok_or_else(|| {
                miette::miette!(
                    "Usage: tangible-objects --convert-device <in.evtest> --to <profile|file.toml>"
                )
            })?;
            let target = convert_to.ok_or_else(|| {
                miette::miette!(
                    "Usage: tangible-objects --convert-device <in.evtest> --to <profile|file.toml>"
                )
            })?;
            modes::run_convert_device(
                Path::new(&input),
                &target,
                device_profiles.as_deref().map(Path::new),
            )
        }
        "--list" => {
            modes::list_devices();
            Ok(())
        }
        "--gui" => {
            let dev = device.ok_or_else(|| {
                miette::miette!("Usage: tangible-objects --gui --device /dev/input/eventX")
            })?;
            let interfaces = parse_interfaces(graphql_listen, mcp_listen, dbus)?;
            let config = modes::DetectionConfig {
                edge,
                fields: modes::load_field_sets(fields_path.as_deref()),
                completion: parse_completion(
                    completion_mode,
                    completion_radius,
                    completion_missing,
                    completion_no_persist,
                    affinity_factor,
                    smoothing_alpha,
                )?,
            };
            run_gui_mode(
                &dev,
                profiles_dir.as_deref(),
                device_profiles.as_deref().map(PathBuf::from),
                config,
                interfaces,
            )
        }
        _ => {
            print_help();
            Ok(())
        }
    }
}

/// Builds the [`tangible_objects_model::CompletionConfig`]: base values
/// come from `~/.config/tangible-objects/completion.toml` (or defaults);
/// each `--completion-*` flag overrides its field.
fn parse_completion(
    mode: Option<String>,
    radius: Option<f64>,
    max_missing: Option<u32>,
    no_persist: bool,
    affinity: Option<f64>,
    smoothing: Option<f64>,
) -> Result<tangible_objects_model::CompletionConfig> {
    use tangible_objects_model::CompletionMode;

    let mut config = modes::load_completion();
    if let Some(value) = mode {
        config.mode = value
            .parse::<CompletionMode>()
            .map_err(|_| miette::miette!("--completion-mode must be one of: match, exact"))?;
    }
    if let Some(factor) = radius {
        config.radius_factor = factor;
    }
    if let Some(missing) = max_missing {
        if !(1..=2).contains(&missing) {
            return Err(miette::miette!("--completion-max-missing must be 1 or 2"));
        }
        config.max_missing = missing;
    }
    if no_persist {
        config.persist = false;
    }
    if let Some(factor) = affinity {
        config.affinity_factor = factor.into();
    }
    if let Some(alpha) = smoothing {
        config.smoothing_alpha = alpha.into();
    }
    Ok(config)
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
    eprintln!("tangible-objects - kapazitive Touchfiguren-Prototyp");
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
        "  tangible-objects --similarity <profiles-dir>                    Profil-Aehnlichkeit / Kollisionsflaeche analysieren"
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
    eprintln!(
        "  tangible-objects --filter --device /dev/input/eventX               Filter-Proxy (Grab + uinput-Klon)"
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
    eprintln!(
        "  --edge-filter-mm <mm>  Geister-Kontakte am Rand verwerfen (--live/--record/--gui)"
    );
    eprintln!("  --affinity-factor <f>  Incumbent-Toleranz-Faktor (Default 2.0, 1.0 = aus)");
    eprintln!("  --smoothing-alpha <a>  Kontakt-Positionsglättung EMA (Default 0.6, 0 = aus)");
    eprintln!("  --fields <file|dir>       Occupancy-Feld-Sets laden (TOML)");
    eprintln!("  --completion-mode <m>     match|exact (Default: match)");
    eprintln!("  --completion-radius <f>   Claim-Radius × Toleranz (Default: 2.0)");
    eprintln!("  --completion-max-missing <n>  fehlende Pins 1|2 (Default: 1)");
    eprintln!("  --completion-no-persist   geclaimte Slots nicht binden");
    eprintln!(
        "  --stability-frames <n> Konsekutive Match-Frames vor Suppression (--filter, Default: 2)"
    );
    eprintln!(
        "  --legacy <auto|always|never>  BTN_TOUCH/ABS_X/Y-Handling (--filter, Default: auto)"
    );
    eprintln!();
    eprintln!("Beispiel:");
    eprintln!("  tangible-objects --replay --file research/figure-tracking-4.txt");
    eprintln!("  tangible-objects --live --device /dev/input/event2");
    eprintln!("  tangible-objects --train --device /dev/input/event2 --name Ritter");
}

/// Runs the GTK visualization (feature `gui`).
#[cfg(feature = "gui")]
fn run_gui_mode(
    dev: &str,
    profiles_dir: Option<&str>,
    device_profiles: Option<PathBuf>,
    config: modes::DetectionConfig,
    interfaces: modes::Interfaces,
) -> Result<()> {
    gtk_app::run_gui(dev, profiles_dir, device_profiles, config, interfaces);
    Ok(())
}

/// Fallback when the binary was built without the `gui` feature.
#[cfg(not(feature = "gui"))]
fn run_gui_mode(
    _dev: &str,
    _profiles_dir: Option<&str>,
    _device_profiles: Option<PathBuf>,
    _config: modes::DetectionConfig,
    _interfaces: modes::Interfaces,
) -> Result<()> {
    eprintln!("GUI-Modus nicht kompiliert. Baue mit: cargo build --features gui");
    std::process::exit(1);
}
