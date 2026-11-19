//! `UnregisterSpellVariables` - the variables of
//! `unregisterSpell`.

use cynic::QueryVariables;
use tangible_objects_model::SpellName;

/// Variables of `unregisterSpell`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct UnregisterSpellVariables {
    /// Spell name to remove.
    pub name: SpellName,
}
