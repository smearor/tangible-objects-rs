use crate::RegistrationRejected;
use crate::SeatAdded;

/// `addSeat` result union - the applied outcome either added the
/// seat to the session's roster or carries the typed rejection
/// (`SESSION_NOT_FOUND`, `SESSION_NOT_OWNED_BY`, `DUPLICATE_SEAT`,
/// `MARKER_PROFILE_NOT_FOUND`).
///
/// No serde derive: GraphQL unions serialize as `__typename`-tagged
/// members - client crates model the wire shape with their own cynic
/// fragments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Union))]
pub enum AddSeatResult {
    /// The seat was added to the session.
    Added(SeatAdded),
    /// The add was rejected - `rejection` says why.
    Rejected(RegistrationRejected),
}
