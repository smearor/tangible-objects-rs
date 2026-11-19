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

mod add_seat_result;
mod claim_seat_session_result;
mod release_seat_session_result;
mod remove_seat_result;
mod seat_added;
mod seat_event;
mod seat_id;
mod seat_marker_state;
mod seat_name;
mod seat_removed;
mod seat_session;
mod seat_session_claimed;
mod seat_session_name;
mod seat_session_released;
mod seat_updated;
mod table_top_seat;
mod tracker;
mod update_seat_result;

pub use add_seat_result::AddSeatResult;
pub use claim_seat_session_result::ClaimSeatSessionResult;
pub use release_seat_session_result::ReleaseSeatSessionResult;
pub use remove_seat_result::RemoveSeatResult;
pub use seat_added::SeatAdded;
pub use seat_event::SeatEvent;
pub use seat_id::SeatId;
pub use seat_marker_state::SeatMarkerState;
pub use seat_name::SeatName;
pub use seat_removed::SeatRemoved;
pub use seat_session::SeatSession;
pub use seat_session_claimed::SeatSessionClaimed;
pub use seat_session_name::SeatSessionName;
pub use seat_session_released::SeatSessionReleased;
pub use seat_updated::SeatUpdated;
pub use table_top_seat::TableTopSeat;
pub use tracker::SeatTracker;
pub use update_seat_result::UpdateSeatResult;
