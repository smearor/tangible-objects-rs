//! `RegisterSpellResult` - the `registerSpell` result union.

use crate::RegistrationRejected;
use crate::SpellRegistered;

/// `registerSpell` result union - the applied outcome either
/// registered the spell or carries the typed rejection.
///
/// No serde derive: GraphQL unions serialize as `__typename`-tagged
/// members - client crates model the wire shape with their own cynic
/// fragments.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::Union))]
pub enum RegisterSpellResult {
    /// The spell was registered.
    Registered(SpellRegistered),
    /// The registration was rejected - `rejection` says why, with
    /// structured detail.
    Rejected(RegistrationRejected),
}
