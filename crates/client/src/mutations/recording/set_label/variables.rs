//! `SetRecordLabelVariables` - the variables of `setRecordLabel`.

use cynic::QueryVariables;

/// Variables of `setRecordLabel`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct SetRecordLabelVariables {
    /// New label; `None` clears it.
    pub label: Option<String>,
}
