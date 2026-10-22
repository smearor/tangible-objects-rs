//! `UnregisterFieldSetVariables` - the variables of
//! `unregisterFieldSet`.

use cynic::QueryVariables;
use tangible_objects_model::FieldSetName;

/// Variables of `unregisterFieldSet`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct UnregisterFieldSetVariables {
    /// Set name to remove.
    pub name: FieldSetName,
}
