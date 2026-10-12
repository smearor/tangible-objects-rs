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
