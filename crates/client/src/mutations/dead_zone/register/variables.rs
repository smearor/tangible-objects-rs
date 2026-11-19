//! `RegisterDeadZoneVariables` - the variables of
//! `registerDeadZone`.

use cynic::QueryVariables;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;

use crate::mutations::dead_zone::register::input::RegisterDeadZoneInput;

/// Variables of `registerDeadZone`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct RegisterDeadZoneVariables {
    /// The zone definition.
    pub input: RegisterDeadZoneInput,
    /// Owning application.
    pub owner: Option<Owner>,
    /// `session` or `persistent`.
    pub lifetime: Option<Lifetime>,
}
