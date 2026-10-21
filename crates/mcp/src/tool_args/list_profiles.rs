//! `ListProfilesArgs` - arguments for the `list_profiles` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::Owner;

/// Arguments for the `list_profiles` tool - mirrors the GraphQL
/// `profiles(owner, staticOnly)` query filters.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListProfilesArgs {
    /// Filter to profiles registered by this owner; null returns all.
    pub owner: Option<Owner>,
    /// Keep only owner-less static profiles (loaded from files).
    pub static_only: Option<bool>,
}
