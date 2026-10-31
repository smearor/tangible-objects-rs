//! `register_field_set()`/`unregister_field_set()` methods of
//! [`GraphQLClient`].

use tangible_objects_model::FieldSet;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::RegisterFieldSetResult;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::field_set::register::mutation::RegisterFieldSet;
use crate::mutations::field_set::register::result::RegisterFieldSetResultFragment;
use crate::mutations::field_set::register::variables::RegisterFieldSetVariables;
use crate::mutations::field_set::unregister::mutation::UnregisterFieldSet;
use crate::mutations::field_set::unregister::variables::UnregisterFieldSetVariables;

impl GraphQLClient {
    /// Registers an occupancy field set (`registerFieldSet`) - the
    /// `FieldSet`'s region/members flatten into the InputObject shape.
    /// The result union carries the typed rejection (e.g. the overlap
    /// detail for `FIELDS_OVERLAP`), not just a message.
    pub async fn register_field_set(
        &self,
        set: &FieldSet,
        owner: Option<&Owner>,
        lifetime: Lifetime,
    ) -> Result<RegisterFieldSetResult, ClientError> {
        let fragment: RegisterFieldSetResultFragment = self
            .call_mutation::<RegisterFieldSet, _, _>(
                "registerFieldSet",
                RegisterFieldSetVariables {
                    input: set.into(),
                    owner: owner.cloned(),
                    lifetime: Some(lifetime),
                },
            )
            .await?;
        fragment.into_result()
    }

    /// Removes a field set (`unregisterFieldSet`).
    pub async fn unregister_field_set(&self, name: &FieldSetName) -> Result<bool, ClientError> {
        self.call_mutation::<UnregisterFieldSet, _, _>(
            "unregisterFieldSet",
            UnregisterFieldSetVariables { name: name.clone() },
        )
        .await
    }
}
