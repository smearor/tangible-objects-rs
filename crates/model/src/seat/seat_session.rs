use crate::Owner;
use crate::SeatSessionName;
use crate::TableTopSeat;
use serde::Deserialize;
use serde::Serialize;

/// An application's seat roster at the tabletop
/// (TANGIBLE_OBJECTS_TABLE_TOP_SEAT.md).
///
/// Claimed by exactly one owner - claim/release semantics mirror
/// profile claims (`ClaimProfile`/`ReleaseProfile`): re-claiming is
/// idempotent for the same owner, a foreign claim is rejected, and
/// `UnregisterOwner`/disconnect cleanup releases the session together
/// with all its seats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "mcp", derive(schemars::JsonSchema))]
pub struct SeatSession {
    /// Session name - unique across the daemon, addresses the session
    /// on all interfaces.
    pub name: SeatSessionName,
    /// Application owning the session - only the owner may mutate its
    /// roster.
    pub owner: Owner,
    /// The session's seats in insertion order.
    pub seats: Vec<TableTopSeat>,
}
