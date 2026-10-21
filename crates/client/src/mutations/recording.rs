//! Recording-session mutations - `startRecording`, `stopRecording`
//! and `setRecordLabel`.

use serde_json::json;
use tangible_objects_model::RecordLabel;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Starts a recording session (`startRecording`) - `base_path` is
    /// the artifact's output path on the daemon host.
    pub async fn start_recording(
        &self,
        base_path: &str,
        label: Option<&RecordLabel>,
    ) -> Result<bool, ClientError> {
        self.call(
            "startRecording",
            "mutation($basePath: String!, $label: String) { startRecording(basePath: $basePath, label: $label) }",
            json!({ "basePath": base_path, "label": label.map(|l| l.as_str()) }),
        )
        .await
    }

    /// Stops the recording session (`stopRecording`).
    pub async fn stop_recording(&self) -> Result<bool, ClientError> {
        self.call("stopRecording", "mutation { stopRecording }", json!({}))
            .await
    }

    /// Sets the ground-truth label mid-session (`setRecordLabel`).
    pub async fn set_record_label(&self, label: Option<&str>) -> Result<bool, ClientError> {
        self.call(
            "setRecordLabel",
            "mutation($label: String) { setRecordLabel(label: $label) }",
            json!({ "label": label }),
        )
        .await
    }
}
