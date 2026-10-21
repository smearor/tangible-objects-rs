//! `AnalyzeRecordingQuery` - the cynic fragment for the
//! `analyzeRecording` query.

use cynic::QueryFragment;
use serde_json::Value;

use crate::queries::analysis::recording::variables::AnalyzeRecordingVariablesFields;

/// Root-field wrapper for `analyzeRecording` - the `JSON` scalar
/// selects as `serde_json::Value`; the response decodes into
/// `AnalysisReport`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "AnalyzeRecordingVariables"
)]
pub(crate) struct AnalyzeRecordingQuery {
    /// `analyzeRecording` field.
    #[cynic(rename = "analyzeRecording")]
    #[arguments(path: $path, mode: $mode, profilesDir: $profiles_dir)]
    analyze_recording: Value,
}
