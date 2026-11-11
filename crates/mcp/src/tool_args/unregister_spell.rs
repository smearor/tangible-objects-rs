//! `UnregisterSpellArgs` - arguments for the `unregister_spell` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::SpellName;

/// Arguments for the `unregister_spell` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnregisterSpellArgs {
    /// Spell name.
    pub name: SpellName,
}
