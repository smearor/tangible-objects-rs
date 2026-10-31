//! `RegisterPattern` - the cynic fragment for the `registerPattern`
//! mutation.

use crate::mutations::pattern::register::result::RegisterPatternResultFragment;
use crate::mutations::pattern::register::variables::RegisterPatternVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `registerPattern`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "RegisterPatternVariables"
)]
pub(crate) struct RegisterPattern {
    /// `registerPattern` field - union decode (`__typename` picks the
    /// member).
    #[cynic(rename = "registerPattern")]
    #[arguments(input: $input, owner: $owner, lifetime: $lifetime)]
    register_pattern: RegisterPatternResultFragment,
}
