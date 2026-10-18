//! Shared device/config loading for the run_* modes: calibration,
//! profile and field-set loading, completion config and device selection.

use std::path::Path;
use std::path::PathBuf;

use miette::IntoDiagnostic;
use miette::Result;
use tangible_objects_engine::TouchDevice;
use tangible_objects_model::Calibration;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::DeviceRegistry;
use tangible_objects_model::FieldSet;
use tangible_objects_model::PatternBuf;
use tangible_objects_model::ProfileFile;
use tracing::warn;

use super::store::runtime_store_dir;

/// Effective calibration for a live device.
///
/// ABS_INFO (`device.calibration()`) is authoritative for axis bounds, but
/// when the device's profile declares a resolution override the measured
/// physical resolution wins - some panels misreport their units/mm in
/// firmware (e.g. the SiS panel reporting 7/12 while driving a 65" display).
/// Returns the reported calibration when the device has no profile or the
/// profile has no override.
pub fn effective_calibration(
    device: &TouchDevice,
    device_profiles: Option<&Path>,
) -> Option<Calibration> {
    let reported = device.calibration()?;
    let mut registry = DeviceRegistry::builtin();
    if let Some(dir) = device_profiles {
        registry = registry.with_dir(dir);
    }
    let id = device.inner().input_id();
    let profile = registry.find_by_ids(id.vendor(), id.product())?;
    let (x_res, y_res) = profile.resolution_override()?;
    println!(
        "Resolution-Override aus Geräteprofil: X={:.3}, Y={:.3} units/mm",
        x_res.as_f64(),
        y_res.as_f64()
    );
    Some(Calibration::with_bounds(
        x_res,
        y_res,
        reported.x_min,
        reported.x_max,
        reported.y_min,
        reported.y_max,
    ))
}

/// Loads figure patterns from a profile directory (default: `profiles/`).
pub fn load_patterns(profiles_dir: Option<&str>) -> PatternBuf {
    let dir = profiles_dir.unwrap_or("profiles");
    let mut dir_result = ProfileFile::load_dir(Path::new(dir));
    // `Lifetime::Persistent` registrations live in the runtime store and
    // reload with the static set.
    if let Some(store) = runtime_store_dir("profiles.d")
        && store.is_dir()
    {
        let mut stored = ProfileFile::load_dir(&store);
        dir_result.profiles.append(&mut stored.profiles);
        dir_result.errors.append(&mut stored.errors);
    }
    for (_, profile) in &dir_result.profiles {
        if profile.enabled {
            println!(
                "Profil geladen: {} ({} Punkte, {} Abstände)",
                profile.name,
                profile.num_points,
                profile.sorted_sides.len()
            );
        } else {
            println!("Profil deaktiviert: {}", profile.name);
        }
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

/// Default path of the completion config file:
/// `$XDG_CONFIG_HOME/tangible-objects/completion.toml`, falling back to
/// `~/.config/tangible-objects/completion.toml`.
pub fn completion_config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("tangible-objects").join("completion.toml"))
}

/// Loads the completion config from [`completion_config_path`]; falls
/// back to [`CompletionConfig::default`] when the file is absent or
/// unreadable. CLI `--completion-*` flags override individual fields on
/// top of this.
pub fn load_completion() -> CompletionConfig {
    let Some(path) = completion_config_path() else {
        return CompletionConfig::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str::<CompletionConfig>(&contents) {
            Ok(config) => {
                println!(
                    "Completion-Config geladen: {} (mode={} radius={} max-missing={} persist={})",
                    path.display(),
                    config.mode,
                    config.radius_factor,
                    config.max_missing,
                    config.persist
                );
                config
            }
            Err(e) => {
                warn!(path = %path.display(), error = %e, "completion config unreadable - defaults used");
                CompletionConfig::default()
            }
        },
        Err(_) => CompletionConfig::default(),
    }
}

/// Loads occupancy field sets from a file or directory (`--fields`).
///
/// A `.toml` file loads one set, a directory loads all sets inside.
/// Invalid files are reported and skipped.
pub fn load_field_sets(path: Option<&str>) -> Vec<FieldSet> {
    let mut sets = Vec::new();
    // Persisted field sets (`Lifetime::Persistent`) reload with the
    // static `--fields` set.
    if let Some(store) = runtime_store_dir("fields.d")
        && store.is_dir()
    {
        let (stored, errors) = FieldSet::load_dir(&store);
        for (file, error) in &errors {
            eprintln!(
                "Warnung: Feld-Set {} konnte nicht geladen werden: {error}",
                file.display()
            );
        }
        sets.extend(stored);
    }
    let Some(path) = path else {
        return sets;
    };
    let path = Path::new(path);
    let file_sets = if path.is_dir() {
        let (file_sets, errors) = FieldSet::load_dir(path);
        for (file, error) in &errors {
            eprintln!(
                "Warnung: Feld-Set {} konnte nicht geladen werden: {error}",
                file.display()
            );
        }
        file_sets
    } else {
        match FieldSet::load(path) {
            Ok(set) => vec![set],
            Err(error) => {
                eprintln!(
                    "Warnung: Feld-Set {} konnte nicht geladen werden: {error}",
                    path.display()
                );
                Vec::new()
            }
        }
    };
    sets.extend(file_sets);
    for set in &sets {
        println!(
            "Feld-Set geladen: {} ({} Felder)",
            set.name,
            set.fields.len()
        );
    }
    sets
}

/// Resolves the device argument: an explicit `--device` path wins;
/// otherwise auto-detect via `vendor:product` against the device registry
/// (builtin + `--device-profiles` dir) - succeeds only when exactly one
/// known touchscreen is attached.
pub fn resolve_device(device: Option<String>, profiles_dir: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = device {
        return Ok(PathBuf::from(path));
    }
    let mut registry = DeviceRegistry::builtin();
    if let Some(dir) = profiles_dir {
        registry = registry.with_dir(dir);
    }
    let entry = TouchDevice::detect(&registry).into_diagnostic()?;
    let profile = registry
        .find_by_ids(entry.vendor, entry.product)
        .expect("detected entry matches a profile");
    println!(
        "Gerät erkannt: {} ({}) → {}",
        profile.name,
        entry.name,
        entry.path.display()
    );
    Ok(entry.path)
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
