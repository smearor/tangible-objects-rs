//! `ListSeatSessionsArgs` - arguments for the `list_seat_sessions`
//! tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::SeatSessionName;

/// Arguments for the `list_seat_sessions` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListSeatSessionsArgs {
    /// Filters to one named session - omitted lists all claimed
    /// sessions.
    #[serde(default)]
    pub session: Option<SeatSessionName>,
}
