//! Analysis queries - `analyzeRecording`, `analyzeSimilarity` and
//! `recordingArtifact` (the artifact stays raw file text).

use serde_json::json;
use tangible_objects_analysis::AnalysisReport;
use tangible_objects_analysis::SimilarityReport;
use tangible_objects_model::AnalysisMode;

use crate::ClientError;
use crate::GraphQLClient;
use crate::client::gql_enum;

impl GraphQLClient {
    /// Deviation analysis over a recording (`analyzeRecording` query -
    /// `Json<AnalysisReport>` passthrough).
    pub async fn analyze_recording(
        &self,
        path: &str,
        mode: AnalysisMode,
        profiles_dir: Option<&str>,
    ) -> Result<AnalysisReport, ClientError> {
        self.call(
            "analyzeRecording",
            "query($path: String!, $mode: AnalysisMode, $profilesDir: String) { analyzeRecording(path: $path, mode: $mode, profilesDir: $profilesDir) }",
            json!({
                "path": path,
                "mode": gql_enum(&mode),
                "profilesDir": profiles_dir,
            }),
        )
        .await
    }

    /// Pairwise profile-collision report (`analyzeSimilarity` query -
    /// `Json<SimilarityReport>` passthrough).
    pub async fn analyze_similarity(
        &self,
        profiles_dir: Option<&str>,
    ) -> Result<SimilarityReport, ClientError> {
        self.call(
            "analyzeSimilarity",
            "query($profilesDir: String) { analyzeSimilarity(profilesDir: $profilesDir) }",
            json!({ "profilesDir": profiles_dir }),
        )
        .await
    }

    /// Recording artifact content (`recordingArtifact` query) - raw
    /// file text, legitimately an untyped `String`.
    pub async fn recording_artifact(&self, path: &str) -> Result<String, ClientError> {
        self.call(
            "recordingArtifact",
            "query($path: String!) { recordingArtifact(path: $path) }",
            json!({ "path": path }),
        )
        .await
    }
}
