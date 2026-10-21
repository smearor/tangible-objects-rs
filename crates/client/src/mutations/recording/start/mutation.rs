//! `StartRecording` - the cynic fragment for the `startRecording`
//! mutation.

use cynic::QueryFragment;

use crate::mutations::recording::start::variables::StartRecordingVariablesFields;

/// Root-field wrapper for `startRecording`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "StartRecordingVariables"
)]
pub(crate) struct StartRecording {
    /// `startRecording` field.
    #[cynic(rename = "startRecording")]
    #[arguments(basePath: $base_path, label: $label)]
    start_recording: bool,
}
