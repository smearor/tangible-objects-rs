//! `AnalyzeArgs` - arguments for the `analyze_recording` tool.

use schemars::JsonSchema;
use serde::Deserialize;

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
