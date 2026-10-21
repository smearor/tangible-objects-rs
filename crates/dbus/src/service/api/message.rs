//! Message methods - `show_message`.

use tangible_objects_runtime::ControlCommand;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// Shows a user-facing message in the GUI overlay; an empty string
/// clears it.
pub(crate) async fn show_message(service: &EngineService, text: &str) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::ShowMessage(text.to_string()))
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
