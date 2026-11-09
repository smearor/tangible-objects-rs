//! `SetBackgroundAlphaArgs` - arguments for the
//! `set_background_alpha` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_background_alpha` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetBackgroundAlphaArgs {
    /// Canvas background opacity in percent, 0-100.
    pub percent: u8,
}
