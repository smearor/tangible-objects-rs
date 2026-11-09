//! `SetFieldsVisibleArgs` - arguments for the
//! `set_fields_visible` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_fields_visible` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetFieldsVisibleArgs {
    /// `true` draws the overlay, `false` hides it.
    pub visible: bool,
}
