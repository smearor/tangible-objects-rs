//! `FilterModeQuery` - the cynic fragment for the `filterMode`
//! query.

use cynic::QueryFragment;
use tangible_objects_model::FilterMode;

/// Root-field wrapper for the `filterMode` query - cynic derives the
/// document and checks the field against `schema.graphql` at compile
/// time.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) struct FilterModeQuery {
    /// `filterMode` field.
    filter_mode: FilterMode,
}
