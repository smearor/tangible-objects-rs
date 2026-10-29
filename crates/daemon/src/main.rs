//! Command-line entry point of the `tangible-objects` daemon binary.
//!
//! Modes: `--list` (enumerate devices), `--replay` (replay an evtest
//! recording), `--live` (live detection), `--record` (frame + raw recording),
//! `--train` (record a figure profile), `--gui` (GTK visualization,
//! feature-gated).

mod modes;

use std::net::SocketAddr;
use std::path::PathBuf;

use crate::modes::Interfaces;
use clap::CommandFactory;
use clap::Parser;
use miette::IntoDiagnostic;
use miette::Result;
use tangible_objects_engine::LegacyMode;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::CompletionMode;
use tangible_objects_model::DetectionConfig;
use tangible_objects_model::Millimeters;
use tangible_objects_model::RecordLabel;
use tangible_objects_model::Region;

/// Parses a `--legacy` value - `LegacyMode`'s `Err` is `()`, which clap
/// cannot render, so the message is supplied here.
fn parse_legacy(value: &str) -> std::result::Result<LegacyMode, String> {
    value
        .parse()
        .map_err(|_| "must be one of: auto, always, never".to_string())
}

/// kapazitive Touchfiguren-Prototyp - Erkennung, Aufzeichnung und
/// Analyse greifbarer Figuren auf Multitouch-Oberflächen.
///
/// Genau ein Modus-Flag wählt die Aktion; alle weiteren Optionen
/// konfigurieren sie. `--record` ist doppeldeutig: als einziges Argument
/// ein eigener Modus, neben `--live`/`--replay` ein Modifier.
#[derive(Parser)]
#[command(name = "tangible-objects", version, about)]
struct Cli {
    /// Replay einer evtest-Aufzeichnung
    #[arg(long, group = "mode")]
    replay: bool,
    /// Live von einem Eingabegerät
    #[arg(long, group = "mode")]
    live: bool,
    /// Eingabegeräte auflisten
    #[arg(long, group = "mode")]
    list: bool,
    /// Profil aufnehmen und speichern (braucht --name)
    #[arg(long, group = "mode")]
    train: bool,
    /// GTK Fullscreen-Visualisierung (Feature gui)
    #[cfg(feature = "gui")]
    #[arg(long, group = "mode")]
    gui: bool,
    /// Filter-Proxy: Grab + uinput-Klon, Tangibles unterdrückt
    #[arg(long, group = "mode")]
    filter: bool,
    /// Aufzeichnung analysieren (Phase 2+3)
    #[arg(long, group = "mode", value_name = "recording.jsonl")]
    analyze: Option<PathBuf>,
    /// Profil-Ähnlichkeit / Kollisionsfläche analysieren
    #[arg(long, group = "mode", value_name = "profiles-dir")]
    similarity: Option<PathBuf>,
    /// Gelabeltes Dataset exportieren (Phase 3)
    #[arg(long, group = "mode", value_name = "recording.jsonl")]
    export_dataset: Option<PathBuf>,
    /// evtest-Aufzeichnung in ein anderes Geräteprofil konvertieren
    #[arg(long, group = "mode", value_name = "in.evtest")]
    convert_device: Option<PathBuf>,

    /// evdev-Gerät (Standard: Auto-Detect über Geräte-Registry)
    #[arg(long, value_name = "/dev/input/eventX")]
    device: Option<String>,
    /// Eingabedatei für --replay
    #[arg(long, value_name = "file")]
    file: Option<PathBuf>,
    /// Profil-Verzeichnis
    #[arg(long, value_name = "dir")]
    profiles: Option<String>,
    /// Figurenname (für --train)
    #[arg(long)]
    name: Option<String>,
    /// Toleranz für Musterabgleich in mm (Standard: 30.0)
    #[arg(long, value_name = "mm")]
    tolerance: Option<f64>,
    /// Aufzeichnungs-Basis: <base>.jsonl + <base>.evtest
    #[arg(long, value_name = "base")]
    record: Option<PathBuf>,
    /// Ground-Truth-Label für --record
    #[arg(long)]
    record_label: Option<String>,
    /// JSON-Report-Datei für --similarity
    #[arg(long, requires = "similarity", value_name = "file")]
    similarity_json: Option<PathBuf>,
    /// Kernel-MT-Slot-State beim Start nicht importieren (offene
    /// Kontakte vor dem Daemon-Start bleiben unsichtbar)
    #[arg(long)]
    no_import_slots: bool,
    /// Zielprofil oder TOML-Datei für --convert-device
    #[arg(long, value_name = "profile|file.toml")]
    to: Option<String>,
    /// Verzeichnis mit Geräteprofilen (TOML)
    #[arg(long, value_name = "dir")]
    device_profiles: Option<PathBuf>,
    /// GraphQL-Endpunkt (z.B. 127.0.0.1:8080)
    #[arg(long, value_name = "addr")]
    graphql_listen: Option<SocketAddr>,
    /// MCP-HTTP-Endpunkt (z.B. 127.0.0.1:8090)
    #[arg(long, value_name = "addr")]
    mcp_listen: Option<SocketAddr>,
    /// D-Bus-Interface (Session-Bus)
    #[arg(long)]
    dbus: bool,
    /// BTN_TOUCH/ABS_X/Y-Handling (--filter)
    #[arg(long, value_name = "auto|always|never", value_parser = parse_legacy)]
    legacy: Option<LegacyMode>,
    /// Konsekutive Match-Frames vor Suppression (--filter, Default: 2)
    #[arg(long, value_name = "n")]
    stability_frames: Option<u64>,
    /// Geister-Kontakte am Rand verwerfen (--live/--record/--gui)
    #[arg(long, value_name = "mm")]
    edge_filter_mm: Option<f64>,
    /// Zone maskieren (x_min,y_min,x_max,y_max in mm, wiederholbar;
    /// --filter)
    #[arg(long, value_name = "x0,y0,x1,y1")]
    zone: Vec<Region>,
    /// Occupancy-Feld-Sets laden (TOML-Datei oder -Verzeichnis)
    #[arg(long, value_name = "file|dir")]
    fields: Option<String>,
    /// Completion-Modus (Default: match)
    #[arg(long, value_name = "match|exact")]
    completion_mode: Option<CompletionMode>,
    /// Claim-Radius × Toleranz (Default: 2.0)
    #[arg(long, value_name = "f")]
    completion_radius: Option<f64>,
    /// Fehlende Pins, die Completion ergänzen darf (Default: 1)
    #[arg(long, value_name = "1|2", value_parser = clap::value_parser!(u32).range(1..=2))]
    completion_max_missing: Option<u32>,
    /// Geclaimte Slots nicht binden
    #[arg(long)]
    completion_no_persist: bool,
    /// Incumbent-Toleranz-Faktor (Default 2.0, 1.0 = aus)
    #[arg(long, value_name = "f")]
    affinity_factor: Option<f64>,
    /// Kontakt-Positionsglättung EMA (Default 0.6, 0 = aus)
    #[arg(long, value_name = "a")]
    smoothing_alpha: Option<f64>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();

    let device_profiles = cli.device_profiles.as_deref();
    let completion = || -> Result<CompletionConfig> {
        let mut config = modes::load_completion();
        if let Some(mode) = cli.completion_mode {
            config.mode = mode;
        }
        if let Some(factor) = cli.completion_radius {
            config.radius_factor = factor;
        }
        if let Some(missing) = cli.completion_max_missing {
            config.max_missing = missing;
        }
        if cli.completion_no_persist {
            config.persist = false;
        }
        if let Some(factor) = cli.affinity_factor {
            config.affinity_factor = factor.into();
        }
        if let Some(alpha) = cli.smoothing_alpha {
            config.smoothing_alpha = alpha.into();
        }
        Ok(config)
    };
    let detection_config = |completion| -> Result<DetectionConfig> {
        Ok(DetectionConfig {
            edge: cli.edge_filter_mm.map(Millimeters::new),
            fields: modes::load_field_sets(cli.fields.as_deref()),
            completion,
            zones: cli.zone.clone(),
            import_slots: !cli.no_import_slots,
        })
    };
    let interfaces = || -> Result<Interfaces> {
        Ok(Interfaces {
            graphql_addr: cli.graphql_listen,
            dbus: cli.dbus,
            mcp_addr: cli.mcp_listen,
        })
    };

    if let Some(path) = &cli.analyze {
        return modes::run_analyze(path, cli.profiles.as_deref());
    }
    if let Some(dir) = &cli.similarity {
        return modes::run_similarity(dir, cli.similarity_json.as_deref());
    }
    if let Some(path) = &cli.export_dataset {
        return modes::run_export_dataset(path);
    }
    if let Some(input) = &cli.convert_device {
        let target = cli.to.clone().ok_or_else(|| {
            miette::miette!(
                "Usage: tangible-objects --convert-device <in.evtest> --to <profile|file.toml>"
            )
        })?;
        return modes::run_convert_device(input, &target, device_profiles);
    }
    if cli.replay {
        let path = cli.file.as_deref().ok_or_else(|| {
            miette::miette!("Usage: tangible-objects --replay --file <evtest-file>")
        })?;
        let patterns = modes::load_patterns(cli.profiles.as_deref());
        return modes::run_replay(
            path,
            (&patterns).into(),
            cli.record.as_deref(),
            cli.record_label.map(RecordLabel::new),
        );
    }
    if cli.filter {
        // Filter proxy: grab the real device and re-emit non-tangible
        // events on a uinput clone (FILTER_PROXY_VIRTUAL.md).
        let dev = modes::resolve_device(cli.device, device_profiles)?;
        let patterns = modes::load_patterns(cli.profiles.as_deref());
        return modes::run_filter(
            &dev,
            (&patterns).into(),
            interfaces()?,
            cli.stability_frames
                .map(tangible_objects_model::FrameCount::new),
            cli.legacy,
            detection_config(completion()?)?,
            device_profiles,
        );
    }
    if cli.train {
        let dev = modes::resolve_device(cli.device, device_profiles)?;
        let fig_name = cli
            .name
            .ok_or_else(|| miette::miette!("Usage: tangible-objects --train --name <name>"))?;
        let tol = Millimeters::new(cli.tolerance.unwrap_or(30.0));
        return modes::run_train(
            &dev,
            &fig_name,
            tol,
            cli.profiles.as_deref(),
            device_profiles,
            !cli.no_import_slots,
        );
    }
    #[cfg(feature = "gui")]
    if cli.gui {
        let dev = cli.device.ok_or_else(|| {
            miette::miette!("Usage: tangible-objects --gui --device /dev/input/eventX")
        })?;
        return run_gui_mode(
            &dev,
            cli.profiles.as_deref(),
            device_profiles.map(PathBuf::from),
            detection_config(completion()?)?,
            interfaces()?,
        );
    }
    if cli.list {
        modes::list_devices();
        return Ok(());
    }
    if cli.live || cli.record.is_some() {
        // Without --device, auto-detect a known touchscreen by
        // vendor:product (builtin + --device-profiles registry).
        let path = modes::resolve_device(cli.device, device_profiles)?;
        let patterns = modes::load_patterns(cli.profiles.as_deref());
        return match &cli.record {
            Some(base) => modes::run_record(
                &path,
                base,
                cli.record_label.map(RecordLabel::new),
                (&patterns).into(),
                interfaces()?,
                detection_config(completion()?)?,
                device_profiles,
            ),
            None => modes::run_live(
                &path,
                (&patterns).into(),
                interfaces()?,
                detection_config(completion()?)?,
                device_profiles,
            ),
        };
    }

    // No mode selected - clap already covers --help; bare invocation
    // prints the same text.
    Cli::command().print_help().into_diagnostic()?;
    println!();
    Ok(())
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
    // Daemon plumbing flows into the embedded GUI as injected hooks -
    // the gtk crate links engine/model/runtime, never the daemon.
    let hooks = tangible_objects_gtk::GuiHooks {
        spawn_interfaces: Box::new(move |handle| modes::spawn_interfaces(handle, &interfaces)),
        drain_control_commands: modes::drain_control_commands,
        status_of_session: modes::status_of_session,
        effective_calibration: modes::effective_calibration,
        register_profile: modes::register_profile_hook,
    };
    tangible_objects_gtk::run_gui(
        dev.to_string(),
        profiles_dir.map(str::to_string),
        modes::load_patterns(profiles_dir),
        device_profiles,
        config,
        true,
        hooks,
    );
    Ok(())
}
