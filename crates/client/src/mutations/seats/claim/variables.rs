//! `ClaimSeatSessionVariables` - the variables of `claimSeatSession`.

use cynic::QueryVariables;
use tangible_objects_model::Owner;
use tangible_objects_model::SeatSessionName;

/// Variables of `claimSeatSession`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct ClaimSeatSessionVariables {
    /// Session name - the roster's identity on all interfaces.
    pub name: SeatSessionName,
    /// Claiming application.
    pub owner: Owner,
}
