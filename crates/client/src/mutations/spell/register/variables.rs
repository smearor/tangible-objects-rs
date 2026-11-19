//! `RegisterSpellVariables` - the variables of `registerSpell`.

use cynic::QueryVariables;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;

use crate::mutations::spell::register::input::RegisterSpellInput;

/// Variables of `registerSpell`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct RegisterSpellVariables {
    /// The spell definition.
    pub input: RegisterSpellInput,
    /// Owning application.
    pub owner: Option<Owner>,
    /// `session` or `persistent`.
    pub lifetime: Option<Lifetime>,
}
