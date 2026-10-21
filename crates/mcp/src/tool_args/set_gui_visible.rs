//! `SetGuiVisibleArgs` - arguments for the `set_gui_visible` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_gui_visible` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetGuiVisibleArgs {
    /// `true` shows the window, `false` hides it.
    pub visible: bool,
}
