//! `SetRecordLabel` - the cynic fragment for the `setRecordLabel`
//! mutation.

use crate::mutations::recording::set_label::variables::SetRecordLabelVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `setRecordLabel`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SetRecordLabelVariables"
)]
pub(crate) struct SetRecordLabel {
    /// `setRecordLabel` field.
    #[cynic(rename = "setRecordLabel")]
    #[arguments(label: $label)]
    set_record_label: bool,
}
