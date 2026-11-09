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
use tangible_objects_gtk::BackgroundAlpha;
use tangible_objects_gtk::GuiHooks;
use tangible_objects_gtk::GuiOptions;
use tangible_objects_gtk::run_gui;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::CompletionMode;
use tangible_objects_model::DetectionConfig;
use tangible_objects_model::DisplayRotation;
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

/// Capacitive touch-figure prototype - detection, recording and
/// analysis of tangible figures on multitouch surfaces.
///
/// Exactly one mode flag selects the action; all other options
/// configure it. `--record` is ambiguous: alone it is a mode of its
/// own, next to `--live`/`--replay` it is a modifier.
#[derive(Parser)]
#[command(name = "tangible-objects", version, about)]
struct Cli {
    /// Replay an evtest recording
    #[arg(long, group = "mode")]
    replay: bool,
    /// Live from an input device
    #[arg(long, group = "mode")]
    live: bool,
    /// List input devices
    #[arg(long, group = "mode")]
    list: bool,
    /// Record and save a profile (requires --name)
    #[arg(long, group = "mode")]
    train: bool,
    /// GTK fullscreen visualization (gui feature)
    #[cfg(feature = "gui")]
    #[arg(long, group = "mode")]
    gui: bool,
    /// Filter proxy: grab + uinput clone, tangibles suppressed
    #[arg(long, group = "mode")]
    filter: bool,
    /// Analyze a recording (phase 2+3)
    #[arg(long, group = "mode", value_name = "recording.jsonl")]
    analyze: Option<PathBuf>,
    /// Analyze profile similarity / collision surface
    #[arg(long, group = "mode", value_name = "profiles-dir")]
    similarity: Option<PathBuf>,
    /// Export a labeled dataset (phase 3)
    #[arg(long, group = "mode", value_name = "recording.jsonl")]
    export_dataset: Option<PathBuf>,
    /// Convert an evtest recording to another device profile
    #[arg(long, group = "mode", value_name = "in.evtest")]
    convert_device: Option<PathBuf>,

    /// evdev device (default: auto-detect via device registry)
    #[arg(long, value_name = "/dev/input/eventX")]
    device: Option<String>,
    /// Input file for --replay
    #[arg(long, value_name = "file")]
    file: Option<PathBuf>,
    /// Profile directory
    #[arg(long, value_name = "dir")]
    profiles: Option<String>,
    /// Figure name (for --train)
    #[arg(long)]
    name: Option<String>,
    /// Pattern-matching tolerance in mm (default: 30.0)
    #[arg(long, value_name = "mm")]
    tolerance: Option<f64>,
    /// Recording base: <base>.jsonl + <base>.evtest; as a --filter
    /// modifier it produces <base>.evtest (input) + <base>.out.evtest
    /// (the OS-visible stream after suppression) instead
    #[arg(long, value_name = "base")]
    record: Option<PathBuf>,
    /// Ground-truth label for --record
    #[arg(long)]
    record_label: Option<String>,
    /// JSON report file for --similarity
    #[arg(long, requires = "similarity", value_name = "file")]
    similarity_json: Option<PathBuf>,
    /// Do not import kernel MT slot state at startup (contacts open
    /// before the daemon starts stay invisible)
    #[arg(long)]
    no_import_slots: bool,
    /// Target profile or TOML file for --convert-device
    #[arg(long, value_name = "profile|file.toml")]
    to: Option<String>,
    /// Directory with device profiles (TOML)
    #[arg(long, value_name = "dir")]
    device_profiles: Option<PathBuf>,
    /// GraphQL endpoint (e.g. 127.0.0.1:8080)
    #[arg(long, value_name = "addr")]
    graphql_listen: Option<SocketAddr>,
    /// MCP HTTP endpoint (e.g. 127.0.0.1:8090)
    #[arg(long, value_name = "addr")]
    mcp_listen: Option<SocketAddr>,
    /// D-Bus interface (session bus)
    #[arg(long)]
    dbus: bool,
    /// BTN_TOUCH/ABS_X/Y handling (--filter)
    #[arg(long, value_name = "auto|always|never", value_parser = parse_legacy)]
    legacy: Option<LegacyMode>,
    /// Consecutive match frames before suppression (--filter, default: 2)
    #[arg(long, value_name = "n")]
    stability_frames: Option<u64>,
    /// Drop ghost contacts near the edge (--live/--record/--gui)
    #[arg(long, value_name = "mm")]
    edge_filter_mm: Option<f64>,
    /// Mask a zone (x_min,y_min,x_max,y_max in mm, repeatable;
    /// --filter)
    #[arg(long, value_name = "x0,y0,x1,y1")]
    zone: Vec<Region>,
    /// Load occupancy field sets (TOML file or directory)
    #[arg(long, value_name = "file|dir")]
    fields: Option<String>,
    /// Completion mode (default: match)
    #[arg(long, value_name = "match|exact")]
    completion_mode: Option<CompletionMode>,
    /// Claim radius x tolerance (default: 2.0)
    #[arg(long, value_name = "f")]
    completion_radius: Option<f64>,
    /// Missing pins completion may add (default: 1)
    #[arg(long, value_name = "1|2", value_parser = clap::value_parser!(u32).range(1..=2))]
    completion_max_missing: Option<u32>,
    /// Do not bind claimed slots
    #[arg(long)]
    completion_no_persist: bool,
    /// Incumbent tolerance factor (default 2.0, 1.0 = off)
    #[arg(long, value_name = "f")]
    affinity_factor: Option<f64>,
    /// Contact position smoothing EMA (default 0.6, 0 = off)
    #[arg(long, value_name = "a")]
    smoothing_alpha: Option<f64>,
    /// Display rotation for the GTK canvas (--gui): quarter turns the
    /// evdev positions get before rendering - needed when the desktop
    /// shows the panel rotated (e.g. GNOME portrait on a landscape
    /// touchscreen)
    #[arg(long, value_name = "0|90|180|270")]
    display_rotation: Option<DisplayRotation>,
    /// Canvas background opacity for the GTK window (--gui) in percent:
    /// 100 = opaque (default), lower values let the desktop show
    /// through the black fill - touches and UI stay opaque
    #[arg(long, value_name = "0-100")]
    background_alpha: Option<u8>,
    /// Draw the dead-zone overlay on the GUI canvas (--gui);
    /// `set_dead_zones_visible` toggles it afterwards
    #[arg(long)]
    dead_zones_visible: bool,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
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
            dead_zones: modes::load_dead_zones(),
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
            cli.record.as_deref(),
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
            GuiOptions {
                visible: true,
                rotation: cli.display_rotation.unwrap_or_default(),
                background_alpha: cli
                    .background_alpha
                    .map(BackgroundAlpha::from_percent)
                    .unwrap_or_default(),
                dead_zones_visible: cli.dead_zones_visible,
            },
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
    config: DetectionConfig,
    options: GuiOptions,
    interfaces: Interfaces,
) -> Result<()> {
    // Daemon plumbing flows into the embedded GUI as injected hooks -
    // the gtk crate links engine/model/runtime, never the daemon.
    let hooks = GuiHooks {
        spawn_interfaces: Box::new(move |handle| modes::spawn_interfaces(handle, &interfaces)),
        drain_control_commands: modes::drain_control_commands,
        status_of_session: modes::status_of_session,
        effective_calibration: modes::effective_calibration,
        register_profile: modes::register_profile_hook,
    };
    run_gui(
        dev.to_string(),
        profiles_dir.map(str::to_string),
        modes::load_patterns(profiles_dir),
        device_profiles,
        config,
        options,
        hooks,
    );
    Ok(())
}
