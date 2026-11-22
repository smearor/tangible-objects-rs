//! `UnregisterHandProfileVariables` - the variables of
//! `unregisterHandProfile`.

use cynic::QueryVariables;
use tangible_objects_model::HandName;

/// Variables of `unregisterHandProfile`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct UnregisterHandProfileVariables {
    /// Hand name to remove.
    pub name: HandName,
}
