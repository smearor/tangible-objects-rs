//! `UnregisterOwnerVariables` - the variables of `unregisterOwner`.

use cynic::QueryVariables;

/// Variables of `unregisterOwner`.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct UnregisterOwnerVariables {
    /// Owning application whose registrations drop.
    pub owner: String,
}
