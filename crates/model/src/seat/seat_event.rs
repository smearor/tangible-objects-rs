use crate::TableTopSeat;
use serde::Serialize;

/// A seat roster or marker-anchor change.
///
/// Derived per frame by [`crate::SeatTracker::update`] (marker
/// binding, pose moves, absence) or emitted when a seat control
/// command applies (add, update, remove, session release). Mirrors
/// the [`crate::SetAction`] vocabulary for the seat domain; carries
/// the full seat so subscribers update their roster without a follow
/// -up query.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SeatEvent {
    /// A seat was added to a session - virtual or marker-backed
    /// (`SEAT_ADDED`).
    Added(TableTopSeat),
    /// An anchored seat's derived pose changed - the bound marker
    /// tangible moved (`SEAT_MOVED`). (Re-)binds report
    /// [`Self::MarkerBound`] instead; the carried seat holds the fresh
    /// pose either way.
    Moved(TableTopSeat),
    /// A seat was removed from its session (`SEAT_REMOVED`) - explicit
    /// removal, session release or owner cleanup.
    Removed(TableTopSeat),
    /// A marker tangible bound to the seat - first bind or rebind
    /// after an absence (`SEAT_MARKER_BOUND`).
    MarkerBound(TableTopSeat),
    /// The seat's marker tangible stopped being tracked
    /// (`SEAT_MARKER_ABSENT`) - the seat keeps its last pose.
    MarkerAbsent(TableTopSeat),
}
