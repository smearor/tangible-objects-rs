//! `AddSeatArgs` - arguments for the `add_seat` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use smearor_commons_layout::SeatPose;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::SeatName;
use tangible_objects_model::SeatSessionName;

/// Arguments for the `add_seat` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddSeatArgs {
    /// Session receiving the seat - must exist and be owned by
    /// `owner`.
    pub session: SeatSessionName,
    /// Owner adding the seat.
    pub owner: Owner,
    /// Seat name - unique within the session.
    pub name: SeatName,
    /// Marker profile anchoring the seat - omit for a virtual seat.
    #[serde(default)]
    pub marker: Option<PatternId>,
    /// Initial pose - the applied pose for virtual seats, the
    /// placeholder until the marker binds for marker-backed seats.
    #[serde(default)]
    pub pose: Option<SeatPose>,
}
