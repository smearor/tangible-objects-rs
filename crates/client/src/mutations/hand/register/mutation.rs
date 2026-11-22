//! `RegisterHandProfile` - the cynic fragment for the
//! `registerHandProfile` mutation.

use crate::mutations::hand::register::result::RegisterHandProfileResultFragment;
use crate::mutations::hand::register::variables::RegisterHandProfileVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `registerHandProfile`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "RegisterHandProfileVariables"
)]
pub(crate) struct RegisterHandProfile {
    /// `registerHandProfile` field - union decode (`__typename`
    /// picks the member).
    #[cynic(rename = "registerHandProfile")]
    #[arguments(input: $input, owner: $owner, lifetime: $lifetime)]
    register_hand_profile: RegisterHandProfileResultFragment,
}
