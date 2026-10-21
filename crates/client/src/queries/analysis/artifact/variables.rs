//! `RecordingArtifactVariables` - the variables of
//! `recordingArtifact`.

use cynic::QueryVariables;

/// Variables of `recordingArtifact`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct RecordingArtifactVariables {
    /// Recording file path.
    pub path: String,
}
