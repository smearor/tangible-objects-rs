//! `UnregisterFieldSet` - the cynic fragment for the
//! `unregisterFieldSet` mutation.

use crate::mutations::field_set::unregister::variables::UnregisterFieldSetVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `unregisterFieldSet`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "UnregisterFieldSetVariables"
)]
pub(crate) struct UnregisterFieldSet {
    /// `unregisterFieldSet` field.
    #[cynic(rename = "unregisterFieldSet")]
    #[arguments(name: $name)]
    unregister_field_set: bool,
}
