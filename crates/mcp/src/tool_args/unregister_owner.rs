//! `UnregisterOwnerArgs` - arguments for the `unregister_owner` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `unregister_owner` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnregisterOwnerArgs {
    /// Owner name whose registrations are removed.
    pub owner: String,
}
