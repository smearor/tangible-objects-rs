//! `UnregisterPattern` - the cynic fragment for the
//! `unregisterPattern` mutation.

use crate::mutations::pattern::unregister::variables::UnregisterPatternVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `unregisterPattern`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "UnregisterPatternVariables"
)]
pub(crate) struct UnregisterPattern {
    /// `unregisterPattern` field.
    #[cynic(rename = "unregisterPattern")]
    #[arguments(id: $id)]
    unregister_pattern: bool,
}
