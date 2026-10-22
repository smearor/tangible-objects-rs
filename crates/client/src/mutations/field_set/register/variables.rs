//! `RegisterFieldSetVariables` - the variables of
//! `registerFieldSet`.

use cynic::QueryVariables;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;

use crate::mutations::field_set::register::input::RegisterFieldSetInput;

/// Variables of `registerFieldSet`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct RegisterFieldSetVariables {
    /// The set definition.
    pub input: RegisterFieldSetInput,
    /// Owning application.
    pub owner: Option<Owner>,
    /// `session` or `persistent`.
    pub lifetime: Option<Lifetime>,
}
