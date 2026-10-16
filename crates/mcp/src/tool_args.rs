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
    /// Whether the filter mode should be enabled.
    pub enabled: bool,
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

/// Arguments for the `set_record_label` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetRecordLabelArgs {
    /// Ground-truth label for subsequent frames; null clears it.
    pub label: Option<String>,
}
