//! `GetPoseArgs` - arguments for the `get_pose` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::TangibleId;

/// Arguments for the `get_pose` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPoseArgs {
    /// Tracking ID of the tangible to look up.
    pub tangible_id: TangibleId,
}
