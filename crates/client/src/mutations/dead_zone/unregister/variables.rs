//! `UnregisterDeadZoneVariables` - the variables of
//! `unregisterDeadZone`.

use cynic::QueryVariables;
use tangible_objects_model::DeadZoneName;

/// Variables of `unregisterDeadZone`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct UnregisterDeadZoneVariables {
    /// Zone name to remove.
    pub name: DeadZoneName,
}
