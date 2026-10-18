//! Offline analysis commands: device conversion, dataset export and the
//! recording/profile analysis reports.

use std::path::Path;

use miette::IntoDiagnostic;
use miette::Result;
use tangible_objects_analysis::CollisionReport;
use tangible_objects_analysis::Dataset;
use tangible_objects_analysis::DeviationReport;
use tangible_objects_analysis::FlappyReport;
use tangible_objects_analysis::MlReport;
use tangible_objects_analysis::Recording;
use tangible_objects_analysis::ScoringReport;
use tangible_objects_analysis::SimilarityReport;
use tangible_objects_analysis::ToleranceReport;
use tangible_objects_analysis::UtilizationReport;
use tangible_objects_engine::DeviceConverter;
use tangible_objects_model::DeviceRegistry;
use tangible_objects_model::ProfileFile;

use super::setup::load_patterns;

/// Converts a `.evtest` recording into the raw event stream of another
/// touchscreen profile (TRAINING.md device conversion).
///
/// `target` resolves against the device registry (builtin + `--device-profiles`
/// dir) or as a TOML file path; the output lands next to the input as
/// `<stem>.to-<profile-name>.evtest`.
pub fn run_convert_device(input: &Path, target: &str, profiles_dir: Option<&Path>) -> Result<()> {
    let mut registry = DeviceRegistry::builtin();
    if let Some(dir) = profiles_dir {
        registry = registry.with_dir(dir);
    }
    for (path, error) in &registry.errors {
        eprintln!(
            "Warnung: Geräteprofil {} konnte nicht geladen werden: {error}",
            path.display()
        );
    }
    let profile = registry.resolve(target).ok_or_else(|| {
        miette::miette!(
            "Geräteprofil '{target}' nicht gefunden (bekannt: {})",
            registry
                .profiles
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let out = input.with_extension(format!(
        "to-{}.evtest",
        profile.name.replace(' ', "-").to_lowercase()
    ));
    let converter = DeviceConverter::new(profile.clone());
    let stats = converter.convert(input, &out).into_diagnostic()?;
    println!(
        "Konvertiert: {} → {} ({} Events: {} übernommen, {} gedroppt, {} geclampt)",
        input.display(),
        out.display(),
        stats.events,
        stats.kept,
        stats.dropped,
        stats.clamped
    );
    Ok(())
}

/// Labeled dataset export of a `.jsonl` recording (TRAINING.md 3.3).
///
/// Writes `<stem>.dataset.csv` next to the recording - one CSV row per
/// candidate of every labeled frame (normalized deviation features +
/// ground-truth target) for external ML tooling.
pub fn run_export_dataset(path: &Path) -> Result<()> {
    let recording = Recording::load(path).into_diagnostic()?;
    let out = path.with_extension("dataset.jsonl");
    let dataset = Dataset::of(&recording);
    dataset.write_jsonl(&out).into_diagnostic()?;
    println!(
        "Dataset: {} ({} rows, {} features)",
        out.display(),
        dataset.rows.len(),
        dataset.feature_width
    );
    if dataset.rows.is_empty() {
        println!("  (keine gelabelten Frames - mit --record-label aufzeichnen)");
    }
    Ok(())
}

/// Offline analysis of a `.jsonl` recording (TRAINING.md phase 2).
///
/// Prints utilization, tolerance suggestions, flappy-contact metrics and the
/// collision matrix for the loaded frames.
pub fn run_analyze(path: &Path, profiles_dir: Option<&str>) -> Result<()> {
    let recording = Recording::load(path).into_diagnostic()?;
    println!(
        "Analyse: {} ({} Frames)\n",
        path.display(),
        recording.frames().len()
    );
    let patterns = load_patterns(profiles_dir);
    print!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        UtilizationReport::analyze(&recording),
        DeviationReport::analyze(&recording),
        DeviationReport::analyze_contacts(&recording, patterns.as_pattern_set()),
        ToleranceReport::analyze(&recording),
        FlappyReport::analyze(&recording),
        CollisionReport::analyze(&recording),
        ScoringReport::analyze(&recording),
        MlReport::analyze(&recording),
    );
    Ok(())
}

/// Prints the pairwise profile-similarity report for a profile
/// directory - the phantom-collision surface (which reduced contact
/// sets of one figure can satisfy another figure's fingerprint).
pub fn run_similarity(dir: &Path, json_out: Option<&Path>) -> Result<()> {
    let loaded = ProfileFile::load_dir(dir);
    for (file, error) in &loaded.errors {
        eprintln!("{}: {error}", file.display());
    }
    let patterns = loaded.into_patterns();
    let report = SimilarityReport::analyze(patterns.as_pattern_set());
    println!(
        "Aehnlichkeits-Analyse: {} ({} Profile)\n",
        dir.display(),
        patterns.len()
    );
    println!("{report}");
    if let Some(path) = json_out {
        let json = serde_json::to_string_pretty(&report).into_diagnostic()?;
        std::fs::write(path, json).into_diagnostic()?;
        println!("Report gespeichert: {}", path.display());
    }
    Ok(())
}
