//! `AddSeat` - the cynic fragment for the `addSeat` mutation.

use crate::mutations::seats::add::result::AddSeatResultFragment;
use crate::mutations::seats::add::variables::AddSeatVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `addSeat`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "AddSeatVariables"
)]
pub(crate) struct AddSeat {
    /// `addSeat` field - union decode (`__typename` picks the member).
    #[cynic(rename = "addSeat")]
    #[arguments(input: $input, owner: $owner)]
    add_seat: AddSeatResultFragment,
}
