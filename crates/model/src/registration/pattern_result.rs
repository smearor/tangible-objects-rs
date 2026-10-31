use crate::PatternRegistered;
use crate::RegistrationRejected;

/// `registerPattern` result union - the applied outcome either
/// registered the profile (carrying its pattern ID) or carries the
/// typed rejection (`FINGERPRINT_COLLISION`, `PATTERN_NOT_FOUND`).
///
/// No serde derive: GraphQL unions serialize as `__typename`-tagged
/// members - client crates model the wire shape with their own cynic
/// fragments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Union))]
pub enum RegisterPatternResult {
    /// The profile was registered.
    Registered(PatternRegistered),
    /// The registration was rejected - `rejection` says why, with
    /// structured detail.
    Rejected(RegistrationRejected),
}
