//! `PendingTap` - a five-finger tap in flight.

use std::time::Instant;

use tangible_objects_model::TouchPoint;

/// A five-finger tap in flight: the simultaneous contact group's
/// down timestamp plus the tap's contacts - their positions become
/// the handprint constellation once the tap completes (SPELLS.md's
/// per-user spells).
pub(super) struct PendingTap {
    /// Earliest contact's down timestamp - the tap must fully lift
    /// within `TAP_TOTAL` of it.
    pub(super) started: Instant,
    /// The tap's contacts - their positions become the handprint
    /// constellation once the tap completes.
    pub(super) contacts: Vec<TouchPoint>,
}
