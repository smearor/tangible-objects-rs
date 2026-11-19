//! Seat domain types - physical places at the tabletop
//! (TANGIBLE_OBJECTS_TABLE_TOP_SEAT.md).
//!
//! A [`SeatSession`] is claimed by one application and holds a roster
//! of [`TableTopSeat`]s. Marker-backed seats anchor to a tracked
//! tangible whose profile acts as the seat marker - the
//! [`SeatTracker`] derives their [`smearor_commons_layout::SeatPose`]
//! per frame; virtual seats carry an application-provided pose. Seats
//! are *places*, not people: who sits there stays in the
//! application's domain.

mod seat_event;
mod seat_id;
mod seat_marker_state;
mod seat_name;
mod seat_session;
mod seat_session_name;
mod table_top_seat;
mod tracker;

pub use seat_event::SeatEvent;
pub use seat_id::SeatId;
pub use seat_marker_state::SeatMarkerState;
pub use seat_name::SeatName;
pub use seat_session::SeatSession;
pub use seat_session_name::SeatSessionName;
pub use table_top_seat::TableTopSeat;
pub use tracker::SeatTracker;
