//! `SetGuiVisible`/`SetDisplayRotation` - the cynic fragments for the
//! GUI mutations.

use crate::mutations::gui::variables::SetDisplayRotationVariablesFields;
use crate::mutations::gui::variables::SetGuiVisibleVariablesFields;
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
