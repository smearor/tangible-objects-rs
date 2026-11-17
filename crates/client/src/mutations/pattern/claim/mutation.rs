//! `ClaimProfile` - the cynic fragment for the `claimProfile` mutation.

use crate::mutations::pattern::claim::result::ClaimProfileResultFragment;
use crate::mutations::pattern::claim::variables::ClaimProfileVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `claimProfile`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "ClaimProfileVariables"
)]
pub(crate) struct ClaimProfile {
    /// `claimProfile` field - union decode (`__typename` picks the
    /// member).
    #[cynic(rename = "claimProfile")]
    #[arguments(id: $id, owner: $owner, enable: $enable)]
    claim_profile: ClaimProfileResultFragment,
}
