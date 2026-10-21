//! `start_recording()`, `stop_recording()` and `set_record_label()`
//! methods of [`GraphQLClient`].

use tangible_objects_model::RecordLabel;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::recording::set_label::mutation::SetRecordLabel;
use crate::mutations::recording::set_label::variables::SetRecordLabelVariables;
use crate::mutations::recording::start::mutation::StartRecording;
use crate::mutations::recording::start::variables::StartRecordingVariables;
use crate::mutations::recording::stop::mutation::StopRecording;

impl GraphQLClient {
    /// Starts a recording session (`startRecording`) - `base_path` is
    /// the artifact's output path on the daemon host.
    pub async fn start_recording(
        &self,
        base_path: &str,
        label: Option<&RecordLabel>,
    ) -> Result<bool, ClientError> {
        self.call_mutation::<StartRecording, _, _>(
            "startRecording",
            StartRecordingVariables {
                base_path: base_path.to_string(),
                label: label.map(|l| l.as_str().to_string()),
            },
        )
        .await
    }

    /// Stops the recording session (`stopRecording`).
    pub async fn stop_recording(&self) -> Result<bool, ClientError> {
        self.call_mutation::<StopRecording, _, _>("stopRecording", ())
            .await
    }

    /// Sets the ground-truth label mid-session (`setRecordLabel`).
    pub async fn set_record_label(&self, label: Option<&str>) -> Result<bool, ClientError> {
        self.call_mutation::<SetRecordLabel, _, _>(
            "setRecordLabel",
            SetRecordLabelVariables {
                label: label.map(str::to_string),
            },
        )
        .await
    }
}
