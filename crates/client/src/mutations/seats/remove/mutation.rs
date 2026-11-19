//! `RemoveSeat` - the cynic fragment for the `removeSeat` mutation.

use crate::mutations::seats::remove::result::RemoveSeatResultFragment;
use crate::mutations::seats::remove::variables::RemoveSeatVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `removeSeat`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "RemoveSeatVariables"
)]
pub(crate) struct RemoveSeat {
    /// `removeSeat` field - union decode (`__typename` picks the
    /// member).
    #[cynic(rename = "removeSeat")]
    #[arguments(id: $id, owner: $owner)]
    remove_seat: RemoveSeatResultFragment,
}
