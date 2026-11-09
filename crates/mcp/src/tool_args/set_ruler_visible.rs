//! `SetRulerVisibleArgs` - arguments for the
//! `set_ruler_visible` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_ruler_visible` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetRulerVisibleArgs {
    /// `true` draws the overlay, `false` hides it.
    pub visible: bool,
}
