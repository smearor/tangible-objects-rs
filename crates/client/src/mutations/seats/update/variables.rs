//! `UpdateSeatVariables` - the variables of `updateSeat`.

use cynic::QueryVariables;
use tangible_objects_model::Owner;
use tangible_objects_model::SeatId;

use crate::mutations::seats::input::UpdateSeatInput;

/// Variables of `updateSeat`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct UpdateSeatVariables {
    /// Globally unique seat ID.
    pub id: SeatId,
    /// The update spec - `None` fields keep their value.
    pub input: UpdateSeatInput,
    /// Owner mutating the seat - must own its session.
    pub owner: Owner,
}
