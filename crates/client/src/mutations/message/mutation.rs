//! `ShowMessage` - the cynic fragment for the `showMessage`
//! mutation.

use crate::mutations::message::variables::ShowMessageVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `showMessage`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "ShowMessageVariables"
)]
pub(crate) struct ShowMessage {
    /// `showMessage` field.
    #[cynic(rename = "showMessage")]
    #[arguments(text: $text)]
    show_message: bool,
}
