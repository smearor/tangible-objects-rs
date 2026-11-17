//! `ReleaseProfile` - the cynic fragment for the `releaseProfile`
//! mutation.

use crate::mutations::pattern::release::result::ReleaseProfileResultFragment;
use crate::mutations::pattern::release::variables::ReleaseProfileVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `releaseProfile`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "ReleaseProfileVariables"
)]
pub(crate) struct ReleaseProfile {
    /// `releaseProfile` field - union decode (`__typename` picks the
    /// member).
    #[cynic(rename = "releaseProfile")]
    #[arguments(id: $id, owner: $owner)]
    release_profile: ReleaseProfileResultFragment,
}
