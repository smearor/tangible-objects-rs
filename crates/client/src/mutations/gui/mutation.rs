//! `SetGuiVisible`/`SetDisplayRotation`/`SetDeadZonesVisible` - the
//! cynic fragments for the GUI mutations.

use crate::mutations::gui::variables::SetBackgroundAlphaVariablesFields;
use crate::mutations::gui::variables::SetDeadZonesVisibleVariablesFields;
use crate::mutations::gui::variables::SetDisplayRotationVariablesFields;
use crate::mutations::gui::variables::SetEdgeMarginVisibleVariablesFields;
use crate::mutations::gui::variables::SetFieldsVisibleVariablesFields;
use crate::mutations::gui::variables::SetGridVisibleVariablesFields;
use crate::mutations::gui::variables::SetGuiVisibleVariablesFields;
use crate::mutations::gui::variables::SetPositionsVisibleVariablesFields;
use crate::mutations::gui::variables::SetRulerVisibleVariablesFields;
use crate::mutations::gui::variables::SetTouchPassthroughVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `setGuiVisible`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetGuiVisibleVariables"
)]
pub(crate) struct SetGuiVisible {
    /// `setGuiVisible` field.
    #[cynic(rename = "setGuiVisible")]
    #[arguments(visible: $visible)]
    set_gui_visible: bool,
}

/// Root-field wrapper for `setDisplayRotation`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetDisplayRotationVariables"
)]
pub(crate) struct SetDisplayRotation {
    /// `setDisplayRotation` field.
    #[cynic(rename = "setDisplayRotation")]
    #[arguments(rotation: $rotation)]
    set_display_rotation: bool,
}

/// Root-field wrapper for `setDeadZonesVisible`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetDeadZonesVisibleVariables"
)]
pub(crate) struct SetDeadZonesVisible {
    /// `setDeadZonesVisible` field.
    #[cynic(rename = "setDeadZonesVisible")]
    #[arguments(visible: $visible)]
    set_dead_zones_visible: bool,
}

/// Root-field wrapper for `setBackgroundAlpha`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetBackgroundAlphaVariables"
)]
pub(crate) struct SetBackgroundAlpha {
    /// `setBackgroundAlpha` field.
    #[cynic(rename = "setBackgroundAlpha")]
    #[arguments(percent: $percent)]
    set_background_alpha: bool,
}

/// Root-field wrapper for `setEdgeMarginVisible`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetEdgeMarginVisibleVariables"
)]
pub(crate) struct SetEdgeMarginVisible {
    /// `setEdgeMarginVisible` field.
    #[cynic(rename = "setEdgeMarginVisible")]
    #[arguments(visible: $visible)]
    set_edge_margin_visible: bool,
}

/// Root-field wrapper for `setFieldsVisible`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetFieldsVisibleVariables"
)]
pub(crate) struct SetFieldsVisible {
    /// `setFieldsVisible` field.
    #[cynic(rename = "setFieldsVisible")]
    #[arguments(visible: $visible)]
    set_fields_visible: bool,
}

/// Root-field wrapper for `setRulerVisible`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetRulerVisibleVariables"
)]
pub(crate) struct SetRulerVisible {
    /// `setRulerVisible` field.
    #[cynic(rename = "setRulerVisible")]
    #[arguments(visible: $visible)]
    set_ruler_visible: bool,
}

/// Root-field wrapper for `setGridVisible`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetGridVisibleVariables"
)]
pub(crate) struct SetGridVisible {
    /// `setGridVisible` field.
    #[cynic(rename = "setGridVisible")]
    #[arguments(visible: $visible)]
    set_grid_visible: bool,
}

/// Root-field wrapper for `setPositionsVisible`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetPositionsVisibleVariables"
)]
pub(crate) struct SetPositionsVisible {
    /// `setPositionsVisible` field.
    #[cynic(rename = "setPositionsVisible")]
    #[arguments(visible: $visible)]
    set_positions_visible: bool,
}

/// Root-field wrapper for `setTouchPassthrough`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetTouchPassthroughVariables"
)]
pub(crate) struct SetTouchPassthrough {
    /// `setTouchPassthrough` field.
    #[cynic(rename = "setTouchPassthrough")]
    #[arguments(enabled: $enabled)]
    set_touch_passthrough: bool,
}
