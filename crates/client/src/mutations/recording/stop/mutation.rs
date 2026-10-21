//! `StopRecording` - the cynic fragment for the `stopRecording`
//! mutation.

use cynic::QueryFragment;

/// Root-field wrapper for `stopRecording`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) struct StopRecording {
    /// `stopRecording` field.
    #[cynic(rename = "stopRecording")]
    stop_recording: bool,
}
