//! `UpdateSeat` - the cynic fragment for the `updateSeat` mutation.

use crate::mutations::seats::update::result::UpdateSeatResultFragment;
use crate::mutations::seats::update::variables::UpdateSeatVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `updateSeat`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "UpdateSeatVariables"
)]
pub(crate) struct UpdateSeat {
    /// `updateSeat` field - union decode (`__typename` picks the
    /// member).
    #[cynic(rename = "updateSeat")]
    #[arguments(id: $id, input: $input, owner: $owner)]
    update_seat: UpdateSeatResultFragment,
}
