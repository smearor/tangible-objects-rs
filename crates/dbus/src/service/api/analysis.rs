//! Analysis methods - `analyze_recording`, `analyze_similarity`.

use std::path::Path;

use tangible_objects_analysis::AnalysisReport;
use tangible_objects_analysis::Recording;
use tangible_objects_analysis::SimilarityReport;
use tangible_objects_model::AnalysisMode;
use tangible_objects_model::PatternBuf;
use tangible_objects_model::ProfileFile;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// Runs the deviation analysis over a recording artifact on the
/// daemon host - same `mode`/`profiles_dir` semantics as the MCP
/// `analyze_recording` tool (`"contacts"` re-evaluates every
/// same-size contact subset of each frame).
pub(crate) async fn analyze_recording(
    _service: &EngineService,
    path: &str,
    mode: &str,
    profiles_dir: &str,
) -> fdo::Result<String> {
    let recording = Recording::load(Path::new(path))
        .map_err(|e| fdo::Error::InvalidArgs(format!("recording unreadable: {e}")))?;
    let mode = if mode.is_empty() {
        AnalysisMode::default()
    } else {
        mode.parse::<AnalysisMode>()
            .map_err(fdo::Error::InvalidArgs)?
    };
    let patterns = if mode == AnalysisMode::Contacts {
        let dir = if profiles_dir.is_empty() {
            "profiles"
        } else {
            profiles_dir
        };
        let patterns = ProfileFile::load_dir(Path::new(dir)).into_patterns();
        if patterns.is_empty() {
            return Err(fdo::Error::InvalidArgs(format!(
                "no profiles loaded from {dir}"
            )));
        }
        patterns
    } else {
        PatternBuf::default()
    };
    let report = AnalysisReport::analyze(&recording, mode, patterns.as_pattern_set());
    serde_json::to_string_pretty(&report).map_err(|e| fdo::Error::Failed(format!("serialize: {e}")))
}

/// Pairwise profile-collision analysis over a profile directory -
/// every pair's minimum achievable max deviation over same-size
/// contact subsets, classified as Safe/Partial/Exact risk.
pub(crate) async fn analyze_similarity(
    _service: &EngineService,
    profiles_dir: &str,
) -> fdo::Result<String> {
    let dir = if profiles_dir.is_empty() {
        "profiles"
    } else {
        profiles_dir
    };
    let patterns = ProfileFile::load_dir(Path::new(dir)).into_patterns();
    if patterns.is_empty() {
        return Err(fdo::Error::InvalidArgs(format!(
            "no profiles loaded from {dir}"
        )));
    }
    let report = SimilarityReport::analyze(patterns.as_pattern_set());
    serde_json::to_string_pretty(&report).map_err(|e| fdo::Error::Failed(format!("serialize: {e}")))
}
