use crate::RegistrationRejected;
use crate::SeatSessionClaimed;

/// `claimSeatSession` result union - the applied outcome either
/// claimed (or created) the session for the owner or carries the
/// typed rejection (`SESSION_ALREADY_OWNED`).
///
/// No serde derive: GraphQL unions serialize as `__typename`-tagged
/// members - client crates model the wire shape with their own cynic
/// fragments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Union))]
pub enum ClaimSeatSessionResult {
    /// The session is now owned by the claiming application.
    Claimed(SeatSessionClaimed),
    /// The claim was rejected - `rejection` says why.
    Rejected(RegistrationRejected),
}
