//! `UnregisterHandProfileArgs` - arguments for the
//! `unregister_hand_profile` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::HandName;

/// Arguments for the `unregister_hand_profile` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnregisterHandProfileArgs {
    /// Hand name.
    pub name: HandName,
}
