use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `get_pose` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPoseArgs {
    /// Tracking ID of the tangible to look up.
    pub tangible_id: u32,
}

/// Arguments for the `set_filter_mode` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetFilterModeArgs {
    /// Suppression policy: `off`, `immediate`, `gated`, `guarded` or
    /// `tangibles-only` (suppresses all contacts).
    pub mode: String,
    /// Optional consecutive-matched-frames gate (e.g. 2 or 3); omit to
    /// keep the current value.
    pub stability: Option<u64>,
}

/// Arguments for the `train_profile` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TrainProfileArgs {
    /// Name of the new figure profile.
    pub name: String,
}

/// Arguments for the `start_recording` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StartRecordingArgs {
    /// Base path of the recording artifacts (`<base>.jsonl` + `<base>.evtest`).
    pub base_path: String,
    /// Ground-truth label applied to recorded frames.
    pub label: Option<String>,
}

/// Arguments for the `set_completion` tool - every field optional, unset
/// fields keep the current value.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetCompletionArgs {
    /// Completion mode: `match` (claim binds for tracking only) or
    /// `exact` (completed partial counts as exact for suppression).
    pub mode: Option<String>,
    /// Claim radius as a multiple of the pattern's scalar tolerance.
    pub radius_factor: Option<f64>,
    /// Maximum missing pins a partial may complete (1 or 2).
    pub max_missing: Option<u32>,
    /// Whether claimed slots enter the tangible's slot binding.
    pub persist: Option<bool>,
    /// Incumbent re-match tolerance widening (`match_score <= factor`,
    /// `1.0` disables).
    pub affinity_factor: Option<f64>,
    /// Per-slot position smoothing alpha (`0.0` disables).
    pub smoothing_alpha: Option<f64>,
}

/// Arguments for the `show_message` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ShowMessageArgs {
    /// Text to display; empty clears the overlay.
    pub text: String,
}

/// Arguments for the `register_pattern` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RegisterPatternArgs {
    /// Figure name.
    pub name: String,
    /// Flat pin coordinates `[x0, y0, x1, y1, ...]` in millimeters.
    pub points: Vec<f64>,
    /// Scalar matching tolerance in millimeters.
    pub tolerance: f64,
    /// Optional owner/application name.
    pub owner: Option<String>,
    /// `session` (default) or `persistent` (written to the runtime
    /// store).
    pub lifetime: Option<String>,
    /// Whether a partial match may spawn this figure
    /// (SPAWN_AS_PARTIAL.md); default `false` (exact-only spawns).
    pub spawn_on_partial: Option<bool>,
    /// Frames a partial spawn must hold before confirmation;
    /// default 15.
    pub spawn_stability: Option<u64>,
}

/// Arguments for the `update_profile` tool - replaces a registered
/// pattern's geometry while keeping its ID.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateProfileArgs {
    /// Pattern ID from `list_profiles`.
    pub pattern_id: u32,
    /// Figure name.
    pub name: String,
    /// Flat pin coordinates `[x0, y0, x1, y1, ...]` in millimeters.
    pub points: Vec<f64>,
    /// Scalar matching tolerance in millimeters.
    pub tolerance: f64,
    /// Whether a partial match may spawn this figure.
    pub spawn_on_partial: Option<bool>,
    /// Frames a partial spawn must hold before confirmation.
    pub spawn_stability: Option<u64>,
}

/// Arguments for the `register_field_set` tool - the set is given as TOML
/// in the same shape as `--fields` files.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RegisterFieldSetArgs {
    /// Field set as TOML (`name`, optional `stability`/`[window]`,
    /// `[[fields]]` cells).
    pub toml: String,
    /// Optional owner/application name.
    pub owner: Option<String>,
    /// `session` (default) or `persistent` (written to the runtime
    /// store).
    pub lifetime: Option<String>,
}

/// Arguments for the `unregister_pattern` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnregisterPatternArgs {
    /// Pattern ID from `list_profiles`.
    pub pattern_id: u32,
}

/// Arguments for the `unregister_field_set` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnregisterFieldSetArgs {
    /// Field-set name.
    pub name: String,
}

/// Arguments for the `unregister_owner` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnregisterOwnerArgs {
    /// Owner name whose registrations are removed.
    pub owner: String,
}

/// Arguments for the `set_record_label` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetRecordLabelArgs {
    /// Ground-truth label for subsequent frames; null clears it.
    pub label: Option<String>,
}

/// Arguments for the `recording_artifact` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecordingArtifactArgs {
    /// Path of the artifact on the daemon host (`*.jsonl`, `*.evtest`,
    /// `*.dataset.jsonl`; no `..` components allowed).
    pub path: String,
}

/// Optional analysis mode for `analyze_recording`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnalyzeArgs {
    /// Path of the recording on the daemon host.
    pub path: String,
    /// `candidates` (default): fold recorded generic candidates.
    /// `contacts`: re-evaluate every same-size contact subset of each
    /// frame against the pattern fingerprints - covers incumbent-tracked
    /// frames that produce no candidates.
    pub mode: Option<tangible_objects_model::AnalysisMode>,
    /// Profile directory on the daemon host for `contacts` mode
    /// (default `profiles/`).
    pub profiles_dir: Option<String>,
}
