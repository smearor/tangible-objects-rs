use async_graphql::Context;
use async_graphql::Object;
use async_graphql::Result;
use tangible_objects_model::TangibleSetState;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;
use tangible_objects_runtime::EngineHandle;
use tangible_objects_runtime::ProfileInfo;

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

    /// Loaded figure profiles.
    async fn profiles(&self, ctx: &Context<'_>) -> Result<Vec<ProfileInfo>> {
        let handle = ctx.data::<EngineHandle>()?;
        Ok(handle.profiles().to_vec())
    }

    /// Attached input device — name and axis calibration. Needed by remote
    /// clients before converting this host's raw recordings (TRAINING.md
    /// device profiles).
    async fn device_info(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<tangible_objects_runtime::DeviceInfo>> {
        Ok(ctx.data::<EngineHandle>()?.snapshot().device)
    }

    /// Live status of the recording session; `null` while nothing records.
    async fn recording_status(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Option<tangible_objects_runtime::RecordingStatus>> {
        Ok(ctx.data::<EngineHandle>()?.snapshot().recording)
    }

    /// Reads a recording artifact produced by this daemon.
    ///
    /// Restricted to known recording extensions (`.jsonl`, `.evtest`,
    /// `.dataset.jsonl`) and paths below the daemon's working directory —
    /// a LAN/debug convenience, not a general file server.
    async fn recording_artifact(&self, path: String) -> Result<String> {
        let path = std::path::PathBuf::from(&path);
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
