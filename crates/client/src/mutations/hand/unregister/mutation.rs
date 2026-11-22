//! `UnregisterHandProfile` - the cynic fragment for the
//! `unregisterHandProfile` mutation.

use crate::mutations::hand::unregister::variables::UnregisterHandProfileVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `unregisterHandProfile`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "UnregisterHandProfileVariables"
)]
pub(crate) struct UnregisterHandProfile {
    /// `unregisterHandProfile` field.
    #[cynic(rename = "unregisterHandProfile")]
    #[arguments(name: $name)]
    unregister_hand_profile: bool,
}
