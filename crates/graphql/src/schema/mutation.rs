use std::path::PathBuf;

use async_graphql::Context;
use async_graphql::Error;
use async_graphql::Object;
use async_graphql::Result;
use tangible_objects_model::RecordLabel;
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

    /// Starts a recording session writing `<base_path>.jsonl` +
    /// `<base_path>.evtest`, optionally labeled.
    async fn start_recording(
        &self,
        ctx: &Context<'_>,
        base_path: String,
        label: Option<String>,
    ) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::StartRecording {
                base_path: PathBuf::from(base_path),
                label: label.map(RecordLabel::new),
            })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Stops the active recording session.
    async fn stop_recording(&self, ctx: &Context<'_>) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::StopRecording)
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Changes the ground-truth label applied to subsequent recorded frames;
    /// `null` clears it.
    async fn set_record_label(&self, ctx: &Context<'_>, label: Option<String>) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::SetRecordLabel(label.map(RecordLabel::new)))
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }
}
