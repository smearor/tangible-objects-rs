//! `SetCompletion` - the cynic fragment for the `setCompletion`
//! mutation.

use crate::mutations::completion::variables::SetCompletionVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `setCompletion`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetCompletionVariables"
)]
pub(crate) struct SetCompletion {
    /// `setCompletion` field.
    #[cynic(rename = "setCompletion")]
    #[arguments(config: $config)]
    set_completion: bool,
}
