//! `RemoveSeatVariables` - the variables of `removeSeat`.

use cynic::QueryVariables;
use tangible_objects_model::Owner;
use tangible_objects_model::SeatId;

/// Variables of `removeSeat`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct RemoveSeatVariables {
    /// Globally unique seat ID.
    pub id: SeatId,
    /// Owner removing the seat - must own its session.
    pub owner: Owner,
}
