//! `UnregisterOwnerArgs` - arguments for the `unregister_owner` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::Owner;

/// Arguments for the `unregister_owner` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnregisterOwnerArgs {
    /// Owner name whose registrations are removed.
    pub owner: Owner,
}
