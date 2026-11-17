//! `ReleaseProfileArgs` - arguments for the `release_profile` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;

/// Arguments for the `release_profile` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReleaseProfileArgs {
    /// Pattern ID from `find_profile`/`list_profiles` - profile names
    /// are not unique.
    pub pattern_id: PatternId,
    /// Owner giving up its claim.
    pub owner: Owner,
}
