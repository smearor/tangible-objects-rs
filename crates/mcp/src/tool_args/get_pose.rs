//! `GetPoseArgs` - arguments for the `get_pose` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `get_pose` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPoseArgs {
    /// Tracking ID of the tangible to look up.
    pub tangible_id: u32,
}
