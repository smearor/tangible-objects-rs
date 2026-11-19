//! `UnregisterSpell` - the cynic fragment for the `unregisterSpell`
//! mutation.

use crate::mutations::spell::unregister::variables::UnregisterSpellVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `unregisterSpell`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "UnregisterSpellVariables"
)]
pub(crate) struct UnregisterSpell {
    /// `unregisterSpell` field.
    #[cynic(rename = "unregisterSpell")]
    #[arguments(name: $name)]
    unregister_spell: bool,
}
