//! `set_gui_visible()` method of [`GraphQLClient`].

use tangible_objects_model::DisplayRotation;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::gui::mutation::SetBackgroundAlpha;
use crate::mutations::gui::mutation::SetDeadZonesVisible;
use crate::mutations::gui::mutation::SetDisplayRotation;
use crate::mutations::gui::mutation::SetEdgeMarginVisible;
use crate::mutations::gui::mutation::SetFieldsVisible;
use crate::mutations::gui::mutation::SetGridVisible;
use crate::mutations::gui::mutation::SetGuiVisible;
use crate::mutations::gui::mutation::SetPositionsVisible;
use crate::mutations::gui::mutation::SetRulerVisible;
use crate::mutations::gui::mutation::SetTouchPassthrough;
use crate::mutations::gui::variables::SetBackgroundAlphaVariables;
use crate::mutations::gui::variables::SetDeadZonesVisibleVariables;
use crate::mutations::gui::variables::SetDisplayRotationVariables;
use crate::mutations::gui::variables::SetEdgeMarginVisibleVariables;
use crate::mutations::gui::variables::SetFieldsVisibleVariables;
use crate::mutations::gui::variables::SetGridVisibleVariables;
use crate::mutations::gui::variables::SetGuiVisibleVariables;
use crate::mutations::gui::variables::SetPositionsVisibleVariables;
use crate::mutations::gui::variables::SetRulerVisibleVariables;
use crate::mutations::gui::variables::SetTouchPassthroughVariables;

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

    /// Shows or hides the edge-margin overlay on the daemon's GUI
    /// canvas (`setEdgeMarginVisible`) - ignored by headless daemons.
    pub async fn set_edge_margin_visible(&self, visible: bool) -> Result<bool, ClientError> {
        self.call_mutation::<SetEdgeMarginVisible, _, _>(
            "setEdgeMarginVisible",
            SetEdgeMarginVisibleVariables { visible },
        )
        .await
    }

    /// Shows or hides the field-set overlay on the daemon's GUI
    /// canvas (`setFieldsVisible`) - ignored by headless daemons.
    pub async fn set_fields_visible(&self, visible: bool) -> Result<bool, ClientError> {
        self.call_mutation::<SetFieldsVisible, _, _>(
            "setFieldsVisible",
            SetFieldsVisibleVariables { visible },
        )
        .await
    }

    /// Shows or hides the ruler overlay on the daemon's GUI canvas
    /// (`setRulerVisible`) - ignored by headless daemons.
    pub async fn set_ruler_visible(&self, visible: bool) -> Result<bool, ClientError> {
        self.call_mutation::<SetRulerVisible, _, _>(
            "setRulerVisible",
            SetRulerVisibleVariables { visible },
        )
        .await
    }

    /// Shows or hides the grid overlay on the daemon's GUI canvas
    /// (`setGridVisible`) - ignored by headless daemons.
    pub async fn set_grid_visible(&self, visible: bool) -> Result<bool, ClientError> {
        self.call_mutation::<SetGridVisible, _, _>(
            "setGridVisible",
            SetGridVisibleVariables { visible },
        )
        .await
    }

    /// Shows or hides the positions overlay on the daemon's GUI
    /// canvas (`setPositionsVisible`) - ignored by headless daemons.
    pub async fn set_positions_visible(&self, visible: bool) -> Result<bool, ClientError> {
        self.call_mutation::<SetPositionsVisible, _, _>(
            "setPositionsVisible",
            SetPositionsVisibleVariables { visible },
        )
        .await
    }

    /// Lets touches pass through the daemon's GTK window to the
    /// desktop beneath, except where GUI elements are hit
    /// (`setTouchPassthrough`) - ignored by headless daemons,
    /// Wayland only.
    pub async fn set_touch_passthrough(&self, enabled: bool) -> Result<bool, ClientError> {
        self.call_mutation::<SetTouchPassthrough, _, _>(
            "setTouchPassthrough",
            SetTouchPassthroughVariables { enabled },
        )
        .await
    }
}
