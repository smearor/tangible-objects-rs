//! `analyze_recording()`, `analyze_similarity()` and
//! `recording_artifact()` methods of [`GraphQLClient`].

use tangible_objects_analysis::AnalysisReport;
use tangible_objects_analysis::SimilarityReport;
use tangible_objects_model::AnalysisMode;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::analysis::artifact::query::RecordingArtifactQuery;
use crate::queries::analysis::artifact::variables::RecordingArtifactVariables;
use crate::queries::analysis::recording::query::AnalyzeRecordingQuery;
use crate::queries::analysis::recording::variables::AnalyzeRecordingVariables;
use crate::queries::analysis::similarity::query::AnalyzeSimilarityQuery;
use crate::queries::analysis::similarity::variables::AnalyzeSimilarityVariables;

impl GraphQLClient {
    /// Deviation analysis over a recording (`analyzeRecording` query -
    /// `Json<AnalysisReport>` passthrough).
    pub async fn analyze_recording(
        &self,
        path: &str,
        mode: AnalysisMode,
        profiles_dir: Option<&str>,
    ) -> Result<AnalysisReport, ClientError> {
        self.call_query::<AnalyzeRecordingQuery, _, _>(
            "analyzeRecording",
            AnalyzeRecordingVariables {
                path: path.to_string(),
                mode: Some(mode),
                profiles_dir: profiles_dir.map(str::to_string),
            },
        )
        .await
    }

    /// Pairwise profile-collision report (`analyzeSimilarity` query -
    /// `Json<SimilarityReport>` passthrough).
    pub async fn analyze_similarity(
        &self,
        profiles_dir: Option<&str>,
    ) -> Result<SimilarityReport, ClientError> {
        self.call_query::<AnalyzeSimilarityQuery, _, _>(
            "analyzeSimilarity",
            AnalyzeSimilarityVariables {
                profiles_dir: profiles_dir.map(str::to_string),
            },
        )
        .await
    }

    /// Recording artifact content (`recordingArtifact` query) - raw
    /// file text, legitimately an untyped `String`.
    pub async fn recording_artifact(&self, path: &str) -> Result<String, ClientError> {
        self.call_query::<RecordingArtifactQuery, _, _>(
            "recordingArtifact",
            RecordingArtifactVariables {
                path: path.to_string(),
            },
        )
        .await
    }
}
