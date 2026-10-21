//! `RecordingStatusQuery` - the cynic fragment for the
//! `recordingStatus` query.

use cynic::QueryFragment;
use tangible_objects_model::RecordingStatus;

/// Root-field wrapper for the `recordingStatus` query - cynic
/// derives the document and checks every selected field against
/// `schema.graphql` at compile time.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) struct RecordingStatusQuery {
    /// `recordingStatus` field.
    #[cynic(rename = "recordingStatus")]
    recording_status: Option<RecordingStatus>,
}
