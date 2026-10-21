//! `TrainProfileArgs` - arguments for the `train_profile` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::PatternName;

/// Arguments for the `train_profile` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TrainProfileArgs {
    /// Name of the new figure profile.
    pub name: PatternName,
}
