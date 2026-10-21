//! `RecordingArtifactArgs` - arguments for the `recording_artifact`
//! tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `recording_artifact` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecordingArtifactArgs {
    /// Path of the artifact on the daemon host (`*.jsonl`, `*.evtest`,
    /// `*.dataset.jsonl`; no `..` components allowed).
    pub path: String,
}
