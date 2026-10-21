//! `ShowMessageVariables` - the variables of `showMessage`.

use cynic::QueryVariables;

/// Variables of `showMessage`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct ShowMessageVariables {
    /// Overlay text.
    pub text: String,
}
