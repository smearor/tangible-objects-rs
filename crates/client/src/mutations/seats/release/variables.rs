//! `ReleaseSeatSessionVariables` - the variables of
//! `releaseSeatSession`.

use cynic::QueryVariables;
use tangible_objects_model::Owner;
use tangible_objects_model::SeatSessionName;

/// Variables of `releaseSeatSession`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct ReleaseSeatSessionVariables {
    /// Session name to release.
    pub name: SeatSessionName,
    /// Owner giving up its claim.
    pub owner: Owner,
}
