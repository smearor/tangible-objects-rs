//! `RegisterHandProfileResult` - the `registerHandProfile` result union.

use crate::HandProfileRegistered;
use crate::RegistrationRejected;

/// `registerHandProfile` result union - the applied outcome either
/// registered the handprint or carries the typed rejection.
///
/// No serde derive: GraphQL unions serialize as `__typename`-tagged
/// members - client crates model the wire shape with their own cynic
/// fragments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Union))]
pub enum RegisterHandProfileResult {
    /// The handprint was registered.
    Registered(HandProfileRegistered),
    /// The registration was rejected - `rejection` says why, with
    /// structured detail.
    Rejected(RegistrationRejected),
}
