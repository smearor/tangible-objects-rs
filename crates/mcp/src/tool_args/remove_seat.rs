//! `RemoveSeatArgs` - arguments for the `remove_seat` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::Owner;
use tangible_objects_model::SeatId;

/// Arguments for the `remove_seat` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveSeatArgs {
    /// Seat to remove - the globally unique ID `add_seat` returned.
    pub seat: SeatId,
    /// Owner removing the seat - must own its session.
    pub owner: Owner,
}
