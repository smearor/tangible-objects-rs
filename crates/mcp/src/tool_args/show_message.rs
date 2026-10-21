//! `ShowMessageArgs` - arguments for the `show_message` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `show_message` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ShowMessageArgs {
    /// Text to display; empty clears the overlay.
    pub text: String,
}
