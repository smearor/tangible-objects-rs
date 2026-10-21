//! `SetRecordLabelArgs` - arguments for the `set_record_label` tool.

use schemars::JsonSchema;
use serde::Deserialize;
use tangible_objects_model::RecordLabel;

/// Arguments for the `set_record_label` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetRecordLabelArgs {
    /// Ground-truth label for subsequent frames; null clears it.
    pub label: Option<RecordLabel>,
}
