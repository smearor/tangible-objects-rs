//! Analysis tools - `analyze_recording`, `analyze_similarity`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use std::path::Path;
use tangible_objects_analysis::AnalysisReport;
use tangible_objects_analysis::Recording;
use tangible_objects_analysis::SimilarityReport;
use tangible_objects_model::AnalysisMode;
use tangible_objects_model::PatternBuf;
use tangible_objects_model::ProfileFile;

use crate::server::engine_mcp::EngineMcp;
use crate::tool_args::AnalyzeArgs;
use crate::tool_args::SimilarityArgs;

/// Offline-analysis tools.
#[tool_router(router = analysis_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Runs the offline deviation analysis over a recording artifact.
    #[tool(
        description = "Analyze a recording (.jsonl): per-pattern per-distance deviation statistics - mean/p99/max deviation, over-tolerance counts and which pin pair most often blocked the match."
    )]
    async fn analyze_recording(
        &self,
        Parameters(args): Parameters<AnalyzeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let recording = Recording::load(Path::new(&args.path))
            .map_err(|e| ErrorData::invalid_params(format!("recording unreadable: {e}"), None))?;
        let mode = args.mode.unwrap_or_default();
        let patterns = if mode == AnalysisMode::Contacts {
            let dir = args.profiles_dir.as_deref().unwrap_or("profiles");
            let loaded = ProfileFile::load_dir(Path::new(dir));
            let patterns = loaded.into_patterns();
            if patterns.is_empty() {
                return Err(ErrorData::invalid_params(
                    format!("no profiles loaded from {dir}"),
                    None,
                ));
            }
            patterns
        } else {
            PatternBuf::default()
        };
        let report = AnalysisReport::analyze(&recording, mode, patterns.as_pattern_set());
        Ok(CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&report)
                .map_err(|e| ErrorData::internal_error(format!("serialize: {e}"), None))?,
        )]))
    }

    /// Pairwise profile-collision analysis over a profile directory.
    #[tool(
        description = "Analyze a profile directory for fingerprint collisions: every profile pair's minimum achievable max deviation over same-size contact subsets, classified as Safe/Partial/Exact risk."
    )]
    async fn analyze_similarity(
        &self,
        Parameters(args): Parameters<SimilarityArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let dir = args.profiles_dir.as_deref().unwrap_or("profiles");
        let loaded = ProfileFile::load_dir(Path::new(dir));
        let patterns = loaded.into_patterns();
        if patterns.is_empty() {
            return Err(ErrorData::invalid_params(
                format!("no profiles loaded from {dir}"),
                None,
            ));
        }
        let report = SimilarityReport::analyze(patterns.as_pattern_set());
        Ok(CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&report)
                .map_err(|e| ErrorData::internal_error(format!("serialize: {e}"), None))?,
        )]))
    }
}
