//! `set_gui_visible()` method of [`GraphQLClient`].

use tangible_objects_model::DisplayRotation;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::gui::mutation::SetBackgroundAlpha;
use crate::mutations::gui::mutation::SetDeadZonesVisible;
use crate::mutations::gui::mutation::SetDisplayRotation;
use crate::mutations::gui::mutation::SetGuiVisible;
use crate::mutations::gui::variables::SetBackgroundAlphaVariables;
use crate::mutations::gui::variables::SetDeadZonesVisibleVariables;
use crate::mutations::gui::variables::SetDisplayRotationVariables;
use crate::mutations::gui::variables::SetGuiVisibleVariables;

impl GraphQLClient {
    /// Shows or hides the daemon's embedded GUI window
    /// (`setGuiVisible`) - ignored by headless daemons.
    pub async fn set_gui_visible(&self, visible: bool) -> Result<bool, ClientError> {
        self.call_mutation::<SetGuiVisible, _, _>(
            "setGuiVisible",
            SetGuiVisibleVariables { visible },
        )
        .await
    }

    /// Quarter-turn the daemon's GUI canvas renders
    /// (`setDisplayRotation`, 0/90/180/270) - needed when the desktop
    /// shows the touchscreen rotated; ignored by headless daemons.
    pub async fn set_display_rotation(
        &self,
        rotation: DisplayRotation,
    ) -> Result<bool, ClientError> {
        self.call_mutation::<SetDisplayRotation, _, _>(
            "setDisplayRotation",
            SetDisplayRotationVariables { rotation },
        )
        .await
    }

    /// Shows or hides the dead-zone overlay on the daemon's GUI
    /// canvas (`setDeadZonesVisible`) - ignored by headless daemons.
    pub async fn set_dead_zones_visible(&self, visible: bool) -> Result<bool, ClientError> {
        self.call_mutation::<SetDeadZonesVisible, _, _>(
            "setDeadZonesVisible",
            SetDeadZonesVisibleVariables { visible },
        )
        .await
    }

    /// Sets the canvas background opacity in percent, 0-100
    /// (`setBackgroundAlpha`) - ignored by headless daemons.
    pub async fn set_background_alpha(&self, percent: u8) -> Result<bool, ClientError> {
        self.call_mutation::<SetBackgroundAlpha, _, _>(
            "setBackgroundAlpha",
            SetBackgroundAlphaVariables {
                percent: i32::from(percent),
            },
        )
        .await
    }
}
