use async_graphql::Context;
use async_graphql::Object;
use async_graphql::Result;
use tangible_objects_runtime::EngineHandle;

use crate::Profile;
use crate::SetInfo;
use crate::Tangible;
use crate::TouchPoint;

/// Read access to the detection state.
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// All currently tracked tangibles.
    async fn tangibles(&self, ctx: &Context<'_>) -> Result<Vec<Tangible>> {
        let handle = ctx.data::<EngineHandle>()?;
        Ok(handle
            .snapshot()
            .tracked
            .iter()
            .map(Tangible::from)
            .collect())
    }

    /// All currently active touch points.
    async fn touch_points(&self, ctx: &Context<'_>) -> Result<Vec<TouchPoint>> {
        let handle = ctx.data::<EngineHandle>()?;
        Ok(handle
            .snapshot()
            .touches
            .iter()
            .map(TouchPoint::from)
            .collect())
    }

    /// Configured Tangible Object Sets with their current activation state.
    async fn sets(&self, ctx: &Context<'_>) -> Result<Vec<SetInfo>> {
        let handle = ctx.data::<EngineHandle>()?;
        let active = handle.snapshot().active_sets;
        Ok(handle
            .sets()
            .iter()
            .map(|set| SetInfo::from((set, active.contains(&set.id))))
            .collect())
    }

    /// Loaded figure profiles.
    async fn profiles(&self, ctx: &Context<'_>) -> Result<Vec<Profile>> {
        let handle = ctx.data::<EngineHandle>()?;
        Ok(handle.profiles().iter().map(Profile::from).collect())
    }
}
