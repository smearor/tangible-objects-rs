//! `RegisterHandProfileVariables` - the variables of
//! `registerHandProfile`.

use cynic::QueryVariables;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;

use crate::mutations::hand::register::input::RegisterHandProfileInput;

/// Variables of `registerHandProfile`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct RegisterHandProfileVariables {
    /// The handprint definition.
    pub input: RegisterHandProfileInput,
    /// Owning application.
    pub owner: Option<Owner>,
    /// `session` or `persistent`.
    pub lifetime: Option<Lifetime>,
}
