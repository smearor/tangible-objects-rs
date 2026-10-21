//! `TrainProfile` - the cynic fragment for the `trainProfile`
//! mutation.

use crate::mutations::training::variables::TrainProfileVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `trainProfile`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "TrainProfileVariables"
)]
pub(crate) struct TrainProfile {
    /// `trainProfile` field.
    #[cynic(rename = "trainProfile")]
    #[arguments(name: $name)]
    train_profile: bool,
}
