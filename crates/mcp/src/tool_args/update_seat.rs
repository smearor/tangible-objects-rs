//! `UpdateSeatArgs` - arguments for the `update_seat` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use smearor_commons_layout::SeatPose;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::SeatId;
use tangible_objects_model::SeatName;

/// Arguments for the `update_seat` tool - omitted fields keep their
/// current value.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateSeatArgs {
    /// Seat to mutate - the globally unique ID `add_seat` returned.
    pub seat: SeatId,
    /// Owner mutating the seat - must own its session.
    pub owner: Owner,
    /// New seat name - omitted keeps the current one.
    #[serde(default)]
    pub name: Option<SeatName>,
    /// Rebinds the seat to another marker profile - omitted keeps the
    /// current marker unless `clear_marker` is set.
    #[serde(default)]
    pub marker: Option<PatternId>,
    /// Clears the marker binding so the seat turns virtual - wins
    /// over `marker` when both are passed.
    #[serde(default)]
    pub clear_marker: Option<bool>,
    /// New pose - omitted keeps the current one. Honored for virtual
    /// seats; a bound marker re-derives the pose on the next frame.
    #[serde(default)]
    pub pose: Option<SeatPose>,
}
