//! Message methods - `message`, `show_message`.

use tangible_objects_runtime::ControlCommand;
use zbus::fdo;

use crate::service::engine_service::EngineService;

/// The user-facing message currently displayed in the GUI overlay;
/// `""` when none is set.
pub(crate) async fn message(service: &EngineService) -> String {
    service.handle.snapshot().message.unwrap_or_default()
}

/// Shows a user-facing message in the GUI overlay; an empty string
/// clears it.
pub(crate) async fn show_message(service: &EngineService, text: &str) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::ShowMessage(text.to_string()))
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
