//! `UnregisterOwner` - the cynic fragment for the `unregisterOwner`
//! mutation.

use crate::mutations::pattern::unregister_owner::variables::UnregisterOwnerVariablesFields;
use cynic::QueryFragment;

/// Root-field wrapper for `unregisterOwner`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "MutationRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "UnregisterOwnerVariables"
)]
pub(crate) struct UnregisterOwner {
    /// `unregisterOwner` field.
    #[cynic(rename = "unregisterOwner")]
    #[arguments(owner: $owner)]
    unregister_owner: bool,
}
