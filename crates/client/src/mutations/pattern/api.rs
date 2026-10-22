//! `register_pattern()`, `update_profile()`, `unregister_pattern()`
//! and `unregister_owner()` methods of [`GraphQLClient`].

use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::ProfileFile;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::pattern::register::mutation::RegisterPattern;
use crate::mutations::pattern::register::variables::RegisterPatternVariables;
use crate::mutations::pattern::unregister::mutation::UnregisterPattern;
use crate::mutations::pattern::unregister::variables::UnregisterPatternVariables;
use crate::mutations::pattern::unregister_owner::mutation::UnregisterOwner;
use crate::mutations::pattern::unregister_owner::variables::UnregisterOwnerVariables;
use crate::mutations::pattern::update::mutation::UpdateProfile;
use crate::mutations::pattern::update::variables::UpdateProfileVariables;

impl GraphQLClient {
    /// Registers a profile at runtime (`registerPattern`) - the
    /// `ProfileFile`'s geometry is flattened into the InputObject's
    /// `[x0, y0, x1, y1, ...]` form.
    pub async fn register_pattern(
        &self,
        profile: &ProfileFile,
        owner: Option<&Owner>,
        lifetime: Lifetime,
    ) -> Result<bool, ClientError> {
        self.call_mutation::<RegisterPattern, _, _>(
            "registerPattern",
            RegisterPatternVariables {
                input: profile.into(),
                owner: owner.cloned(),
                lifetime: Some(lifetime),
            },
        )
        .await
    }

    /// Replaces a registered pattern's geometry (`updateProfile`).
    pub async fn update_profile(
        &self,
        id: PatternId,
        profile: &ProfileFile,
    ) -> Result<bool, ClientError> {
        self.call_mutation::<UpdateProfile, _, _>(
            "updateProfile",
            UpdateProfileVariables {
                id,
                input: profile.into(),
            },
        )
        .await
    }

    /// Removes a registered pattern (`unregisterPattern`).
    pub async fn unregister_pattern(&self, id: PatternId) -> Result<bool, ClientError> {
        self.call_mutation::<UnregisterPattern, _, _>(
            "unregisterPattern",
            UnregisterPatternVariables { id },
        )
        .await
    }

    /// Drops every registration of `owner` (`unregisterOwner`).
    pub async fn unregister_owner(&self, owner: &Owner) -> Result<bool, ClientError> {
        self.call_mutation::<UnregisterOwner, _, _>(
            "unregisterOwner",
            UnregisterOwnerVariables {
                owner: owner.clone(),
            },
        )
        .await
    }
}
