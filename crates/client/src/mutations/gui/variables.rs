//! `SetGuiVisibleVariables`/`SetDisplayRotationVariables` - the
//! variables of the GUI mutations.

use cynic::QueryVariables;
use tangible_objects_model::DisplayRotation;

/// Variables of `setGuiVisible`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct SetGuiVisibleVariables {
    /// `true` shows the window.
    pub visible: bool,
}

/// Variables of `setDisplayRotation`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct SetDisplayRotationVariables {
    /// Quarter turns the canvas renders.
    pub rotation: DisplayRotation,
}

/// Variables of `setDeadZonesVisible`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct SetDeadZonesVisibleVariables {
    /// `true` draws the dead-zone overlay.
    pub visible: bool,
}

/// Variables of `setBackgroundAlpha`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct SetBackgroundAlphaVariables {
    /// Canvas background opacity in percent, 0-100.
    pub percent: i32,
}
