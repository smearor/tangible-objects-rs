//! Owner methods - `unregister_owner`.

use tangible_objects_model::Owner;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// Removes every pattern and field set owned by `owner` - app
/// cleanup hook; automatic cleanup also fires when a D-Bus caller's
/// unique name vanishes.
pub(crate) async fn unregister_owner(service: &EngineService, owner: Owner) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::UnregisterOwner { owner })
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
