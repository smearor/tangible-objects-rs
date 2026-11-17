use crate::ProfileClaimed;
use crate::RegistrationRejected;

/// `claimProfile` result union - the applied outcome either claimed
/// the profile for the owner (carrying its pattern ID) or carries the
/// typed rejection (`PATTERN_NOT_FOUND`, `PROFILE_ALREADY_OWNED`).
///
/// No serde derive: GraphQL unions serialize as `__typename`-tagged
/// members - client crates model the wire shape with their own cynic
/// fragments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Union))]
pub enum ClaimProfileResult {
    /// The profile is now owned by the claiming application.
    Claimed(ProfileClaimed),
    /// The claim was rejected - `rejection` says why.
    Rejected(RegistrationRejected),
}
