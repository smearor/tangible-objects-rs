//! `MetricsQuery` - the cynic fragment for the `metrics` query.

use cynic::QueryFragment;
use serde_json::Value;

/// Root-field wrapper for the `metrics` query - the `JSON` scalar
/// selects as `serde_json::Value`; the response still decodes into
/// `SessionMetrics` through its `Deserialize`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) struct MetricsQuery {
    /// `metrics` field.
    metrics: Value,
}
