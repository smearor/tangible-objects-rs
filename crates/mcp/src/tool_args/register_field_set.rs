//! `RegisterFieldSetArgs` - arguments for the `register_field_set`
//! tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `register_field_set` tool - the set is given as TOML
/// in the same shape as `--fields` files.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RegisterFieldSetArgs {
    /// Field set as TOML (`name`, optional `stability`/`[window]`,
    /// `[[fields]]` cells).
    pub toml: String,
    /// Optional owner/application name.
    pub owner: Option<String>,
    /// `session` (default) or `persistent` (written to the runtime
    /// store).
    pub lifetime: Option<String>,
}
