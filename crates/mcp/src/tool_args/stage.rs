//! `StageArgs` - arguments for the `register_stage`/`unregister_stage`
//! tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `register_stage`/`unregister_stage` tools.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StageArgs {
    /// Stage scope name.
    pub name: String,
}
