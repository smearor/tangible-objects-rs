use async_graphql::Context;
use async_graphql::Object;
use async_graphql::Result;
use std::path::Path;
use std::path::PathBuf;
use tangible_objects_analysis::AnalysisReport;
use tangible_objects_analysis::Recording;
use tangible_objects_analysis::SimilarityReport;
use tangible_objects_model::AnalysisMode;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternBuf;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::SessionMetrics;
use tangible_objects_model::TangibleSetState;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;
use tangible_objects_runtime::DeviceInfo;
use tangible_objects_runtime::EngineHandle;
use tangible_objects_runtime::ProfileInfo;
use tangible_objects_runtime::RecordingStatus;

/// Read access to the detection state.
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// All currently tracked tangibles.
    async fn tangibles(&self, ctx: &Context<'_>) -> Result<Vec<TrackedTangible>> {
        let handle = ctx.data::<EngineHandle>()?;
        Ok(handle.snapshot().tracked)
    }

    /// All currently active touch points.
    async fn touch_points(&self, ctx: &Context<'_>) -> Result<Vec<TouchPoint>> {
        let handle = ctx.data::<EngineHandle>()?;
        Ok(handle.snapshot().touches)
    }

    /// Configured Tangible Object Sets with their current activation state.
    async fn sets(&self, ctx: &Context<'_>) -> Result<Vec<TangibleSetState>> {
        let handle = ctx.data::<EngineHandle>()?;
        let active = handle.snapshot().active_sets;
        Ok(handle
            .sets()
            .iter()
            .map(|set| TangibleSetState::from((set, active.contains(&set.id))))
            .collect())
    }

    /// Loaded plus dynamically registered figure profiles. `owner`
    /// filters to the registrations of one application; `staticOnly`
    /// filters to profiles without an owner (loaded at startup).
    async fn profiles(
        &self,
        ctx: &Context<'_>,
        owner: Option<String>,
        static_only: Option<bool>,
    ) -> Result<Vec<ProfileInfo>> {
        let handle = ctx.data::<EngineHandle>()?;
        Ok(handle
            .profiles()
            .into_iter()
            .filter(|info| match &owner {
                Some(owner) => info.owner.as_ref().map(Owner::as_str) == Some(owner.as_str()),
                None => true,
            })
            .filter(|info| !(static_only.unwrap_or(false) && info.owner.is_some()))
            .collect())
    }

    /// Attached input device - name and axis calibration. Needed by remote
    /// clients before converting this host's raw recordings (TRAINING.md
    /// device profiles).
    async fn device_info(&self, ctx: &Context<'_>) -> Result<Option<DeviceInfo>> {
        Ok(ctx.data::<EngineHandle>()?.snapshot().device)
    }

    /// Live status of the recording session; `null` while nothing records.
    async fn recording_status(&self, ctx: &Context<'_>) -> Result<Option<RecordingStatus>> {
        Ok(ctx.data::<EngineHandle>()?.snapshot().recording)
    }

    /// Active missing-pin completion configuration
    /// (TRACKING_STABILIZATION.md) - `null` when the device loop has not
    /// published one yet.
    async fn completion(&self, ctx: &Context<'_>) -> Result<Option<CompletionConfig>> {
        Ok(ctx.data::<EngineHandle>()?.snapshot().completion)
    }

    /// Accumulated session recognition metrics: uptime, re-ID churn,
    /// match quality, per-slot contact stability.
    async fn metrics(&self, ctx: &Context<'_>) -> Result<async_graphql::Json<SessionMetrics>> {
        Ok(async_graphql::Json(
            ctx.data::<EngineHandle>()?.snapshot().metrics.clone(),
        ))
    }

    /// User-facing message currently shown by the GUI.
    async fn message(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        Ok(ctx.data::<EngineHandle>()?.snapshot().message)
    }

    /// Reads a recording artifact produced by this daemon.
    ///
    /// Restricted to known recording extensions (`.jsonl`, `.evtest`,
    /// `.dataset.jsonl`) and paths below the daemon's working directory -
    /// a LAN/debug convenience, not a general file server.
    /// Runs the deviation analysis over a recording artifact - same
    /// `mode`/`profiles_dir` semantics as the MCP `analyze_recording`
    /// tool (`"contacts"` re-evaluates every same-size contact subset).
    async fn analyze_recording(
        &self,
        path: String,
        mode: Option<AnalysisMode>,
        profiles_dir: Option<String>,
    ) -> Result<async_graphql::Json<AnalysisReport>> {
        let recording = Recording::load(Path::new(&path))
            .map_err(|e| async_graphql::Error::new(format!("recording unreadable: {e}")))?;
        let mode = mode.unwrap_or_default();
        let patterns = if mode == AnalysisMode::Contacts {
            let dir = profiles_dir.as_deref().unwrap_or("profiles");
            let patterns = ProfileFile::load_dir(Path::new(dir)).into_patterns();
            if patterns.is_empty() {
                return Err(async_graphql::Error::new(format!(
                    "no profiles loaded from {dir}"
                )));
            }
            patterns
        } else {
            PatternBuf::default()
        };
        Ok(async_graphql::Json(AnalysisReport::analyze(
            &recording,
            mode,
            patterns.as_pattern_set(),
        )))
    }

    /// Pairwise profile-collision analysis over a profile directory -
    /// every pair's minimum achievable max deviation over same-size
    /// contact subsets, classified as Safe/Partial/Exact risk.
    async fn analyze_similarity(
        &self,
        profiles_dir: Option<String>,
    ) -> Result<async_graphql::Json<SimilarityReport>> {
        let dir = profiles_dir.as_deref().unwrap_or("profiles");
        let patterns = ProfileFile::load_dir(Path::new(dir)).into_patterns();
        if patterns.is_empty() {
            return Err(async_graphql::Error::new(format!(
                "no profiles loaded from {dir}"
            )));
        }
        Ok(async_graphql::Json(SimilarityReport::analyze(
            patterns.as_pattern_set(),
        )))
    }

    async fn recording_artifact(&self, path: String) -> Result<String> {
        let path = PathBuf::from(&path);
        let valid_ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "jsonl" || e == "evtest")
            .unwrap_or(false);
        let valid_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with(".dataset.jsonl"))
            .unwrap_or(false);
        if path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
            || !(valid_ext || valid_name)
        {
            return Err(async_graphql::Error::new(
                "only recording artifacts (*.jsonl, *.evtest, *.dataset.jsonl) without '..' are readable",
            ));
        }
        std::fs::read_to_string(&path)
            .map_err(|e| async_graphql::Error::new(format!("cannot read {}: {e}", path.display())))
    }
}
