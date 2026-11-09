//! GUI methods - `set_gui_visible`, `set_display_rotation`,
//! `set_dead_zones_visible`.

use crate::service::engine_service::EngineService;
use tangible_objects_model::DisplayRotation;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;

/// Shows or hides the daemon-embedded GUI window (`--gui` mode) -
/// ignored on headless daemons.
pub(crate) async fn set_gui_visible(service: &EngineService, visible: bool) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::SetGuiVisible(visible))
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}

/// Quarter-turn the GUI canvas renders (`--gui` mode): `"0"`, `"90"`,
/// `"180"` or `"270"` degrees clockwise from the viewer's perspective -
/// ignored on headless daemons.
pub(crate) async fn set_display_rotation(
    service: &EngineService,
    rotation: &str,
) -> fdo::Result<()> {
    let parsed: DisplayRotation = rotation.parse().map_err(fdo::Error::InvalidArgs)?;
    service
        .handle
        .send_command(ControlCommand::SetDisplayRotation(parsed))
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}

/// Shows or hides the dead-zone overlay on the GUI canvas - ignored
/// on headless daemons.
pub(crate) async fn set_dead_zones_visible(
    service: &EngineService,
    visible: bool,
) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::SetDeadZonesVisible(visible))
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}

/// Sets the GUI canvas background opacity in percent, 0-100 - ignored
/// on headless daemons.
pub(crate) async fn set_background_alpha(service: &EngineService, percent: u8) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::SetBackgroundAlpha(percent.into()))
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
