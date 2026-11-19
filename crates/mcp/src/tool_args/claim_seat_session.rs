//! `ClaimSeatSessionArgs` - arguments for the `claim_seat_session`
//! tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::Owner;
use tangible_objects_model::SeatSessionName;

/// Arguments for the `claim_seat_session` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClaimSeatSessionArgs {
    /// Session to claim - created when no session of this name
    /// exists.
    pub name: SeatSessionName,
    /// Claiming application - re-claiming is idempotent for the same
    /// owner, rejected for a foreign one.
    pub owner: Owner,
}
