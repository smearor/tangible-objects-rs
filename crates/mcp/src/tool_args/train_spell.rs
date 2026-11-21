//! `TrainSpellArgs` - arguments for the `train_spell` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::SpellName;

/// Arguments for the `train_spell` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TrainSpellArgs {
    /// Name of the spell to train.
    pub name: SpellName,
    /// Arming the finished spell carries - the `[arming]` table as
    /// TOML (e.g. `type = "five-finger-tap"` or
    /// `type = "start-spot"` + `[arming.region]`); `null`/empty =
    /// `free`.
    pub arming: Option<String>,
}
