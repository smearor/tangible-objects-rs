//! `UpdateProfile` - the cynic fragment for the `updateProfile`
//! mutation.

use crate::mutations::pattern::update::variables::UpdateProfileVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `updateProfile`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "UpdateProfileVariables"
)]
pub(crate) struct UpdateProfile {
    /// `updateProfile` field.
    #[cynic(rename = "updateProfile")]
    #[arguments(id: $id, input: $input)]
    update_profile: bool,
}
