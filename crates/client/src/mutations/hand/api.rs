//! `register_hand_profile()`/`unregister_hand_profile()` methods of
//! [`GraphQLClient`].

use tangible_objects_model::HandName;
use tangible_objects_model::HandProfile;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::RegisterHandProfileResult;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::hand::register::mutation::RegisterHandProfile;
use crate::mutations::hand::register::result::RegisterHandProfileResultFragment;
use crate::mutations::hand::register::variables::RegisterHandProfileVariables;
use crate::mutations::hand::unregister::mutation::UnregisterHandProfile;
use crate::mutations::hand::unregister::variables::UnregisterHandProfileVariables;

impl GraphQLClient {
    /// Registers a five-contact handprint (`registerHandProfile`) -
    /// `five-finger-tap` arming can bind spells to it (SPELLS.md's
    /// per-user spells). The result union carries the typed
    /// rejection, not just a message.
    pub async fn register_hand_profile(
        &self,
        hand: &HandProfile,
        owner: Option<&Owner>,
        lifetime: Lifetime,
    ) -> Result<RegisterHandProfileResult, ClientError> {
        let fragment: RegisterHandProfileResultFragment = self
            .call_mutation::<RegisterHandProfile, _, _>(
                "registerHandProfile",
                RegisterHandProfileVariables {
                    input: hand.into(),
                    owner: owner.cloned(),
                    lifetime: Some(lifetime),
                },
            )
            .await?;
        fragment.into_result()
    }

    /// Removes a handprint (`unregisterHandProfile`).
    pub async fn unregister_hand_profile(&self, name: &HandName) -> Result<bool, ClientError> {
        self.call_mutation::<UnregisterHandProfile, _, _>(
            "unregisterHandProfile",
            UnregisterHandProfileVariables { name: name.clone() },
        )
        .await
    }
}
