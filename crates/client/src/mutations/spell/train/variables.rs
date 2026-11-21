//! `TrainSpellVariables` - the variables of `trainSpell`.

use cynic::QueryVariables;
use tangible_objects_model::SpellName;

use crate::mutations::spell::register::arming_input::SpellArmingInput;

/// Variables of `trainSpell` - the spell name plus the finished
/// spell's arming trigger.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct TrainSpellVariables {
    /// Spell name to train.
    pub name: SpellName,
    /// Arming the finished spell carries; `None` = `free`.
    pub arming: Option<SpellArmingInput>,
}
