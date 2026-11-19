//! `SetSeatsVisibleArgs` - arguments for the `set_seats_visible`
//! tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_seats_visible` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetSeatsVisibleArgs {
    /// `true` draws the overlay, `false` hides it.
    pub visible: bool,
}
