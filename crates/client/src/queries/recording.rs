//! `recordingStatus` query - the daemon's recording session state.

use serde_json::json;
use tangible_objects_model::RecordingStatus;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Recording session status (`recordingStatus` query).
    pub async fn recording_status(&self) -> Result<Option<RecordingStatus>, ClientError> {
        self.call(
            "recordingStatus",
            "{ recordingStatus { active label recordedFrames droppedFrames droppedEvents } }",
            json!({}),
        )
        .await
    }
}
