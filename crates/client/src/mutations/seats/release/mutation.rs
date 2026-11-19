//! `ReleaseSeatSession` - the cynic fragment for the
//! `releaseSeatSession` mutation.

use crate::mutations::seats::release::result::ReleaseSeatSessionResultFragment;
use crate::mutations::seats::release::variables::ReleaseSeatSessionVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `releaseSeatSession`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "ReleaseSeatSessionVariables"
)]
pub(crate) struct ReleaseSeatSession {
    /// `releaseSeatSession` field - union decode (`__typename` picks
    /// the member).
    #[cynic(rename = "releaseSeatSession")]
    #[arguments(name: $name, owner: $owner)]
    release_seat_session: ReleaseSeatSessionResultFragment,
}
