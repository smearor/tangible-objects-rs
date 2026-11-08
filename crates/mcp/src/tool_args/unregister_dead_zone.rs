//! `UnregisterDeadZoneArgs` - arguments for the
//! `unregister_dead_zone` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::DeadZoneName;

/// Arguments for the `unregister_dead_zone` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnregisterDeadZoneArgs {
    /// Dead-zone name.
    pub name: DeadZoneName,
}
