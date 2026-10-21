//! `RegisterStage` - the cynic fragment for the `registerStage`
//! mutation.

use crate::mutations::stage::variables::StageVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `registerStage`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "StageVariables"
)]
pub(crate) struct RegisterStage {
    /// `registerStage` field.
    #[cynic(rename = "registerStage")]
    #[arguments(name: $name)]
    register_stage: bool,
}
