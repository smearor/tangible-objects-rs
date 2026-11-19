use crate::RegistrationRejected;
use crate::SeatRemoved;

/// `removeSeat` result union - the applied outcome either removed
/// the seat from its session or carries the typed rejection
/// (`SEAT_NOT_FOUND`, `SESSION_NOT_OWNED_BY`).
///
/// No serde derive: GraphQL unions serialize as `__typename`-tagged
/// members - client crates model the wire shape with their own cynic
/// fragments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Union))]
pub enum RemoveSeatResult {
    /// The seat was removed from its session.
    Removed(SeatRemoved),
    /// The removal was rejected - `rejection` says why.
    Rejected(RegistrationRejected),
}
