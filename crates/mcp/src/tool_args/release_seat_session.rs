//! `ReleaseSeatSessionArgs` - arguments for the
//! `release_seat_session` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::Owner;
use tangible_objects_model::SeatSessionName;

/// Arguments for the `release_seat_session` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReleaseSeatSessionArgs {
    /// Session to release - its seats are removed with it.
    pub name: SeatSessionName,
    /// Owner giving up its claim - only the current owner may
    /// release.
    pub owner: Owner,
}
