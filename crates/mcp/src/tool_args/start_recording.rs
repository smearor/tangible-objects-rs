//! `StartRecordingArgs` - arguments for the `start_recording` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `start_recording` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StartRecordingArgs {
    /// Base path of the recording artifacts (`<base>.jsonl` + `<base>.evtest`).
    pub base_path: String,
    /// Ground-truth label applied to recorded frames.
    pub label: Option<String>,
}
