//! `RegisterFieldSet` - the cynic fragment for the
//! `registerFieldSet` mutation.

use crate::mutations::field_set::register::variables::RegisterFieldSetVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `registerFieldSet`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "RegisterFieldSetVariables"
)]
pub(crate) struct RegisterFieldSet {
    /// `registerFieldSet` field.
    #[cynic(rename = "registerFieldSet")]
    #[arguments(input: $input, owner: $owner, lifetime: $lifetime)]
    register_field_set: bool,
}
