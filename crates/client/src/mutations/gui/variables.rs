//! `SetGuiVisibleVariables` - the variables of `setGuiVisible`.

use cynic::QueryVariables;

/// Variables of `setGuiVisible`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct SetGuiVisibleVariables {
    /// `true` shows the window.
    pub visible: bool,
}
