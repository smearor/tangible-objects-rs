//! `UnregisterPatternArgs` - arguments for the `unregister_pattern`
//! tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::PatternId;

/// Arguments for the `unregister_pattern` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnregisterPatternArgs {
    /// Pattern ID from `list_profiles`.
    pub pattern_id: PatternId,
}
