//! `UnregisterFieldSetArgs` - arguments for the
//! `unregister_field_set` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `unregister_field_set` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnregisterFieldSetArgs {
    /// Field-set name.
    pub name: String,
}
