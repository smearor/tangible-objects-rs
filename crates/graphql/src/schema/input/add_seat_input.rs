//! `AddSeatInput` - the `addSeat` input object: session, seat name,
//! optional marker profile and optional initial pose.

use smearor_commons_layout::SeatPose;
use tangible_objects_model::PatternId;
use tangible_objects_model::SeatName;
use tangible_objects_model::SeatSessionName;

/// Input for `addSeat`.
#[derive(async_graphql::InputObject)]
pub(crate) struct AddSeatInput {
    /// Session receiving the seat - must exist and be owned by the
    /// mutating owner.
    pub(crate) session: SeatSessionName,
    /// Seat name - unique within the session.
    pub(crate) name: SeatName,
    /// Marker profile anchoring the seat - `null` creates a virtual
    /// seat (`pose` is then its applied pose; `null` pose falls back
    /// to the session-default pose).
    pub(crate) marker: Option<PatternId>,
    /// Initial pose - the applied pose for virtual seats, the
    /// placeholder until the marker binds for marker-backed seats.
    pub(crate) pose: Option<SeatPose>,
}
