//! Training methods - `train_profile`.

use tangible_objects_runtime::ControlCommand;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// Starts a training session capturing a new profile with `name`.
pub(crate) async fn train_profile(service: &EngineService, name: &str) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::TrainProfile(name.to_string()))
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
