//! `TrainSpell` - the cynic fragment for the `trainSpell` mutation.

use cynic::QueryFragment;

use crate::mutations::spell::train::variables::TrainSpellVariablesFields;

/// Root-field wrapper for `trainSpell`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "TrainSpellVariables"
)]
pub(crate) struct TrainSpell {
    /// `trainSpell` field.
    #[cynic(rename = "trainSpell")]
    #[arguments(name: $name, arming: $arming)]
    train_spell: bool,
}
