//! `ResetMetrics` - the cynic fragment for the `resetMetrics`
//! mutation.

use cynic::QueryFragment;

/// Root-field wrapper for `resetMetrics`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) struct ResetMetrics {
    /// `resetMetrics` field.
    #[cynic(rename = "resetMetrics")]
    reset_metrics: bool,
}
