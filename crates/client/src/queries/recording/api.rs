//! `recording_status()` method of [`GraphQLClient`].

use tangible_objects_model::RecordingStatus;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::recording::query::RecordingStatusQuery;

impl GraphQLClient {
    /// Live recording status (`recordingStatus` query; `None` when no
    /// session ran).
    pub async fn recording_status(&self) -> Result<Option<RecordingStatus>, ClientError> {
        self.call_query::<RecordingStatusQuery, _, _>("recordingStatus", ())
            .await
    }
}
