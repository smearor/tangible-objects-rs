//! `FieldSetsQuery` - the cynic fragment for the `fieldSets` query.

use cynic::QueryFragment;
use tangible_objects_model::FieldSet;

/// Root-field wrapper for the `fieldSets` query - cynic derives the
/// document and checks every selected field against `schema.graphql`
/// at compile time.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) struct FieldSetsQuery {
    /// `fieldSets` field.
    field_sets: Vec<FieldSet>,
}
