//! `ClaimSeatSession` - the cynic fragment for the `claimSeatSession`
//! mutation.

use crate::mutations::seats::claim::result::ClaimSeatSessionResultFragment;
use crate::mutations::seats::claim::variables::ClaimSeatSessionVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `claimSeatSession`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "ClaimSeatSessionVariables"
)]
pub(crate) struct ClaimSeatSession {
    /// `claimSeatSession` field - union decode (`__typename` picks
    /// the member).
    #[cynic(rename = "claimSeatSession")]
    #[arguments(name: $name, owner: $owner)]
    claim_seat_session: ClaimSeatSessionResultFragment,
}
