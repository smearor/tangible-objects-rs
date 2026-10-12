use async_graphql::Context;
use async_graphql::Error;
use async_graphql::Object;
use async_graphql::Result;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::EngineHandle;

/// Control operations forwarded to the daemon's device loop.
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Enables or disables the tangible filter mode.
    async fn set_filter_mode(&self, ctx: &Context<'_>, enabled: bool) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::SetFilterMode(enabled))
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Starts a training session that captures a new profile with the given
    /// name.
    async fn train_profile(&self, ctx: &Context<'_>, name: String) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::TrainProfile(name))
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }
}
