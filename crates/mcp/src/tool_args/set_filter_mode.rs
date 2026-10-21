//! `SetFilterModeArgs` - arguments for the `set_filter_mode` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_filter_mode` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetFilterModeArgs {
    /// Suppression policy: `off`, `immediate`, `gated`, `guarded` or
    /// `tangibles-only` (suppresses all contacts).
    pub mode: String,
    /// Optional consecutive-matched-frames gate (e.g. 2 or 3); omit to
    /// keep the current value.
    pub stability: Option<u64>,
    /// Optional stage scope - narrows the switch to that stage only
    /// (EVENT_PIPELINE.md); omit to apply to all tangible stages.
    pub stage: Option<String>,
}
