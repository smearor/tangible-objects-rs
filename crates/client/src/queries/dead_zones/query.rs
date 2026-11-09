//! `DeadZonesQuery`/`DeadZoneFilteredQuery` - the cynic fragments
//! for the dead-zone queries.

use cynic::QueryFragment;
use tangible_objects_model::DeadZone;

/// Root-field wrapper for the `deadZones` query - cynic derives the
/// document and checks every selected field against `schema.graphql`
/// at compile time.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) struct DeadZonesQuery {
    /// `deadZones` field.
    dead_zones: Vec<DeadZone>,
}

/// Root-field wrapper for the `deadZoneFiltered` query.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) struct DeadZoneFilteredQuery {
    /// `deadZoneFiltered` field.
    dead_zone_filtered: i32,
}
