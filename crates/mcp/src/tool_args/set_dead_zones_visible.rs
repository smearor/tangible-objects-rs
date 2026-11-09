//! `SetDeadZonesVisibleArgs` - arguments for the
//! `set_dead_zones_visible` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_dead_zones_visible` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetDeadZonesVisibleArgs {
    /// `true` draws the dead-zone overlay, `false` hides it.
    pub visible: bool,
}
