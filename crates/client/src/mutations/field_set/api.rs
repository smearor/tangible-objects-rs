//! `register_field_set()`/`unregister_field_set()` methods of
//! [`GraphQLClient`].

use tangible_objects_model::FieldSet;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::field_set::register::mutation::RegisterFieldSet;
use crate::mutations::field_set::register::variables::RegisterFieldSetVariables;
use crate::mutations::field_set::unregister::mutation::UnregisterFieldSet;
use crate::mutations::field_set::unregister::variables::UnregisterFieldSetVariables;

impl GraphQLClient {
    /// Registers an occupancy field set (`registerFieldSet`) - the
    /// `FieldSet`'s region/members flatten into the InputObject shape.
    pub async fn register_field_set(
        &self,
        set: &FieldSet,
        owner: Option<&Owner>,
        lifetime: Lifetime,
    ) -> Result<bool, ClientError> {
        self.call_mutation::<RegisterFieldSet, _, _>(
            "registerFieldSet",
            RegisterFieldSetVariables {
                input: set.into(),
                owner: owner.map(|o| o.as_str().to_string()),
                lifetime: Some(lifetime),
            },
        )
        .await
    }

    /// Removes a field set (`unregisterFieldSet`).
    pub async fn unregister_field_set(&self, name: &str) -> Result<bool, ClientError> {
        self.call_mutation::<UnregisterFieldSet, _, _>(
            "unregisterFieldSet",
            UnregisterFieldSetVariables {
                name: name.to_string(),
            },
        )
        .await
    }
}
