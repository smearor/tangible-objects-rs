//! GUI methods - `set_gui_visible`.

use tangible_objects_runtime::ControlCommand;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// Shows or hides the daemon-embedded GUI window (`--gui` mode) -
/// ignored on headless daemons.
pub(crate) async fn set_gui_visible(service: &EngineService, visible: bool) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::SetGuiVisible(visible))
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
