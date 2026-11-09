//! `SetPositionsVisibleArgs` - arguments for the
//! `set_positions_visible` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_positions_visible` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetPositionsVisibleArgs {
    /// `true` draws the overlay, `false` hides it.
    pub visible: bool,
}
