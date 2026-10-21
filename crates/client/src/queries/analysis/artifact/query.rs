//! `RecordingArtifactQuery` - the cynic fragment for the
//! `recordingArtifact` query.

use crate::queries::analysis::artifact::variables::RecordingArtifactVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `recordingArtifact`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "RecordingArtifactVariables"
)]
pub(crate) struct RecordingArtifactQuery {
    /// `recordingArtifact` field.
    #[cynic(rename = "recordingArtifact")]
    #[arguments(path: $path)]
    recording_artifact: String,
}
