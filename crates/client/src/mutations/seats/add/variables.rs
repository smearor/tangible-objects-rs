//! `AddSeatVariables` - the variables of `addSeat`.

use cynic::QueryVariables;
use tangible_objects_model::Owner;

use crate::mutations::seats::input::AddSeatInput;

/// Variables of `addSeat`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct AddSeatVariables {
    /// The seat spec - session, name, optional marker and pose.
    pub input: AddSeatInput,
    /// Owner adding the seat - must own the session.
    pub owner: Owner,
}
