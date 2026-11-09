//! `SetTouchPassthroughArgs` - arguments for the
//! `set_touch_passthrough` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_touch_passthrough` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetTouchPassthroughArgs {
    /// `true` lets touches fall through to the desktop except on GUI
    /// elements, `false` disables.
    pub enabled: bool,
}
