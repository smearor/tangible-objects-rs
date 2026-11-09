//! `SetEdgeMarginVisibleArgs` - arguments for the
//! `set_edge_margin_visible` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_edge_margin_visible` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetEdgeMarginVisibleArgs {
    /// `true` draws the overlay, `false` hides it.
    pub visible: bool,
}
