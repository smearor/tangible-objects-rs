//! `SetDisplayRotationArgs` - arguments for the `set_display_rotation` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_display_rotation` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDisplayRotationArgs {
    /// Quarter turns applied to the GUI canvas: `0`, `90`, `180` or
    /// `270` degrees (clockwise from the viewer's perspective).
    pub rotation: String,
}
