//! `RegisterSpellArgs` - arguments for the `register_spell` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::Owner;

/// Arguments for the `register_spell` tool - the spell is given as
/// TOML in the same shape as `spells.d` files.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RegisterSpellArgs {
    /// Spell as TOML (`name`, `fingers`, `corridor`, `[[strokes]]`
    /// plus the `[arming]` section).
    pub toml: String,
    /// Optional owner/application name.
    pub owner: Option<Owner>,
    /// `session` (default) or `persistent` (written to the runtime
    /// store). String + parse on purpose: `Lifetime` enum serde flips
    /// with the `cynic` feature.
    pub lifetime: Option<String>,
}
