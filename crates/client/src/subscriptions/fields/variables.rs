//! `FieldEventsVariables` - the variables of `fieldEvents`.

use cynic::QueryVariables;
use tangible_objects_model::FieldSetName;

/// Variables of `fieldEvents` - `set` narrows to one field set.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct FieldEventsVariables {
    /// Optional set-name filter.
    pub set: Option<FieldSetName>,
}
