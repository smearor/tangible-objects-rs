//! `RegisterSpell` - the cynic fragment for the `registerSpell`
//! mutation.

use crate::mutations::spell::register::result::RegisterSpellResultFragment;
use crate::mutations::spell::register::variables::RegisterSpellVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `registerSpell`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "RegisterSpellVariables"
)]
pub(crate) struct RegisterSpell {
    /// `registerSpell` field - union decode (`__typename` picks the
    /// member).
    #[cynic(rename = "registerSpell")]
    #[arguments(input: $input, owner: $owner, lifetime: $lifetime)]
    register_spell: RegisterSpellResultFragment,
}
