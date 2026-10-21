//! `SetRecordLabelArgs` - arguments for the `set_record_label` tool.

use schemars::JsonSchema;
use serde::Deserialize;

/// Arguments for the `set_record_label` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetRecordLabelArgs {
    /// Ground-truth label for subsequent frames; null clears it.
    pub label: Option<String>,
}
