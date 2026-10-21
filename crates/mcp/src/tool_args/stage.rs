//! `StageArgs` - arguments for the `register_stage`/`unregister_stage`
//! tools.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::StageName;

/// Arguments for the `register_stage`/`unregister_stage` tools.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StageArgs {
    /// Stage scope name.
    pub name: StageName,
}
