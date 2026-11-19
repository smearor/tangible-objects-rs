//! `UpdateSeatInput` - the `updateSeat` input object: every field is
//! optional; `clearMarker` resolves the marker's keep/set/clear
//! tri-state that `Option<PatternId>` alone cannot express.

use smearor_commons_layout::SeatPose;
use tangible_objects_model::PatternId;
use tangible_objects_model::SeatName;

/// Input for `updateSeat` - `null` fields keep their current value.
#[derive(async_graphql::InputObject)]
pub(crate) struct UpdateSeatInput {
    /// New seat name - `null` keeps the current one.
    pub(crate) name: Option<SeatName>,
    /// Rebinds the seat to another marker profile - `null` keeps the
    /// current marker unless `clearMarker` is set.
    pub(crate) marker: Option<PatternId>,
    /// Clears the marker binding so the seat turns virtual - wins
    /// over `marker` when both are passed.
    pub(crate) clear_marker: Option<bool>,
    /// New pose - `null` keeps the current one. Honored for virtual
    /// seats; a bound marker re-derives the pose on the next frame.
    pub(crate) pose: Option<SeatPose>,
}
