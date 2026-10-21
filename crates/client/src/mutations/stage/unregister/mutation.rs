//! `UnregisterStage` - the cynic fragment for the `unregisterStage`
//! mutation.

use crate::mutations::stage::variables::StageVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `unregisterStage`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "StageVariables"
)]
pub(crate) struct UnregisterStage {
    /// `unregisterStage` field.
    #[cynic(rename = "unregisterStage")]
    #[arguments(name: $name)]
    unregister_stage: bool,
}
