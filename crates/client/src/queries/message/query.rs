//! `MessageQuery` - the cynic fragment for the `message` query.

use cynic::QueryFragment;
/// Root-field wrapper for the `message` query.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) struct MessageQuery {
    /// `message` field.
    message: Option<String>,
}
