//! `RegisterPatternVariables` - the variables of `registerPattern`.

use crate::mutations::pattern::input::RegisterPatternInput;
use cynic::QueryVariables;
use tangible_objects_model::Lifetime;

/// Variables of `registerPattern`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct RegisterPatternVariables {
    /// The pattern geometry.
    pub input: RegisterPatternInput,
    /// Owning application.
    pub owner: Option<String>,
    /// `session` or `persistent`.
    pub lifetime: Option<Lifetime>,
}
