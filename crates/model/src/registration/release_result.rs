use crate::ProfileReleased;
use crate::RegistrationRejected;

/// `releaseProfile` result union - the applied outcome either
/// released the profile back to owner-less state or carries the typed
/// rejection (`PATTERN_NOT_FOUND`, `PROFILE_NOT_OWNED_BY`).
///
/// No serde derive: GraphQL unions serialize as `__typename`-tagged
/// members - client crates model the wire shape with their own cynic
/// fragments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Union))]
pub enum ReleaseProfileResult {
    /// The profile is owner-less again.
    Released(ProfileReleased),
    /// The release was rejected - `rejection` says why.
    Rejected(RegistrationRejected),
}
