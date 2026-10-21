//! `AnalyzeRecordingVariables` - the variables of `analyzeRecording`.

use tangible_objects_model::AnalysisMode;
use cynic::QueryVariables;


/// Variables of `analyzeRecording` - `path` is required, `mode` and
/// `profilesDir` optional.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct AnalyzeRecordingVariables {
    /// Recording file path.
    pub path: String,
    /// Analysis pass (`Candidates` default).
    pub mode: Option<AnalysisMode>,
    /// Profiles directory for collision analysis.
    pub profiles_dir: Option<String>,
}
