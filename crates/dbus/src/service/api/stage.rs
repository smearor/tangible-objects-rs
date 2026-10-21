//! Stage methods - `register_stage`, `unregister_stage`.

use tangible_objects_model::StageName;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// Claims or creates a pipeline stage for `name` (EVENT_PIPELINE.md
/// phase 7) - persists until `UnregisterStage`.
pub(crate) async fn register_stage(service: &EngineService, name: StageName) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::RegisterStage { name })
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}

/// Removes the pipeline stage for `name` - its suppression state
/// dies with it.
pub(crate) async fn unregister_stage(service: &EngineService, name: StageName) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::UnregisterStage { name })
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
