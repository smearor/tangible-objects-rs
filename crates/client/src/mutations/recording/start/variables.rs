//! `StartRecordingVariables` - the variables of `startRecording`.

use cynic::QueryVariables;

/// Variables of `startRecording`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct StartRecordingVariables {
    /// Artifact output path on the daemon host.
    pub base_path: String,
    /// Ground-truth label, if any.
    pub label: Option<String>,
}
