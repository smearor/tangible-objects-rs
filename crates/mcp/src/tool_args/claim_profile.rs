//! `ClaimProfileArgs` - arguments for the `claim_profile` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;

/// Arguments for the `claim_profile` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClaimProfileArgs {
    /// Pattern ID from `find_profile`/`list_profiles` - profile names
    /// are not unique.
    pub pattern_id: PatternId,
    /// Claiming application.
    pub owner: Owner,
    /// Activate the profile in the same operation (default: false).
    #[serde(default)]
    pub enable: bool,
}
