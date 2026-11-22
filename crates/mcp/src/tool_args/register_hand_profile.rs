//! `RegisterHandProfileArgs` - arguments for the
//! `register_hand_profile` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::Owner;

/// Arguments for the `register_hand_profile` tool - the handprint is
/// given as TOML in the same shape as `hands.d` files.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RegisterHandProfileArgs {
    /// Hand profile as TOML (`name`, `tolerance`, `points`).
    pub toml: String,
    /// Optional owner/application name.
    pub owner: Option<Owner>,
    /// `session` (default) or `persistent` (written to the runtime
    /// store). String + parse on purpose: `Lifetime` enum serde flips
    /// with the `cynic` feature.
    pub lifetime: Option<String>,
}
