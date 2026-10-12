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
}
