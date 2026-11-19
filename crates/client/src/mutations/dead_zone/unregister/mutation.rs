//! `UnregisterDeadZone` - the cynic fragment for the
//! `unregisterDeadZone` mutation.

use crate::mutations::dead_zone::unregister::variables::UnregisterDeadZoneVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `unregisterDeadZone`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "UnregisterDeadZoneVariables"
)]
pub(crate) struct UnregisterDeadZone {
    /// `unregisterDeadZone` field.
    #[cynic(rename = "unregisterDeadZone")]
    #[arguments(name: $name)]
    unregister_dead_zone: bool,
}
