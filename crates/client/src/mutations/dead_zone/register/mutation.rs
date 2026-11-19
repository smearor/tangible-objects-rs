//! `RegisterDeadZone` - the cynic fragment for the
//! `registerDeadZone` mutation.

use crate::mutations::dead_zone::register::result::RegisterDeadZoneResultFragment;
use crate::mutations::dead_zone::register::variables::RegisterDeadZoneVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `registerDeadZone`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "RegisterDeadZoneVariables"
)]
pub(crate) struct RegisterDeadZone {
    /// `registerDeadZone` field - union decode (`__typename` picks
    /// the member).
    #[cynic(rename = "registerDeadZone")]
    #[arguments(input: $input, owner: $owner, lifetime: $lifetime)]
    register_dead_zone: RegisterDeadZoneResultFragment,
}
