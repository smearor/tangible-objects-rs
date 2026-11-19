use crate::RegistrationRejected;
use crate::SeatUpdated;

/// `updateSeat` result union - the applied outcome either mutated
/// the seat or carries the typed rejection (`SEAT_NOT_FOUND`,
/// `SESSION_NOT_OWNED_BY`, `DUPLICATE_SEAT`,
/// `MARKER_PROFILE_NOT_FOUND`).
///
/// No serde derive: GraphQL unions serialize as `__typename`-tagged
/// members - client crates model the wire shape with their own cynic
/// fragments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Union))]
pub enum UpdateSeatResult {
    /// The seat was updated.
    Updated(SeatUpdated),
    /// The update was rejected - `rejection` says why.
    Rejected(RegistrationRejected),
}
