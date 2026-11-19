use crate::RegistrationRejected;
use crate::SeatSessionReleased;

/// `releaseSeatSession` result union - the applied outcome either
/// released the session (its seats went with it) or carries the
/// typed rejection (`SESSION_NOT_FOUND`, `SESSION_NOT_OWNED_BY`).
///
/// No serde derive: GraphQL unions serialize as `__typename`-tagged
/// members - client crates model the wire shape with their own cynic
/// fragments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Union))]
pub enum ReleaseSeatSessionResult {
    /// The session was released and its seats removed.
    Released(SeatSessionReleased),
    /// The release was rejected - `rejection` says why.
    Rejected(RegistrationRejected),
}
