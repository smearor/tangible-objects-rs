//! `register_pattern()`, `update_profile()`, `unregister_pattern()`,
//! `claim_profile()`, `release_profile()` and `unregister_owner()`
//! methods of [`GraphQLClient`].

use tangible_objects_model::ClaimProfileResult;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::RegisterPatternResult;
use tangible_objects_model::ReleaseProfileResult;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::pattern::claim::mutation::ClaimProfile;
use crate::mutations::pattern::claim::result::ClaimProfileResultFragment;
use crate::mutations::pattern::claim::variables::ClaimProfileVariables;
use crate::mutations::pattern::register::mutation::RegisterPattern;
use crate::mutations::pattern::register::result::RegisterPatternResultFragment;
use crate::mutations::pattern::register::variables::RegisterPatternVariables;
use crate::mutations::pattern::release::mutation::ReleaseProfile;
use crate::mutations::pattern::release::result::ReleaseProfileResultFragment;
use crate::mutations::pattern::release::variables::ReleaseProfileVariables;
use crate::mutations::pattern::unregister::mutation::UnregisterPattern;
use crate::mutations::pattern::unregister::variables::UnregisterPatternVariables;
use crate::mutations::pattern::unregister_owner::mutation::UnregisterOwner;
use crate::mutations::pattern::unregister_owner::variables::UnregisterOwnerVariables;
use crate::mutations::pattern::update::mutation::UpdateProfile;
use crate::mutations::pattern::update::variables::UpdateProfileVariables;

impl GraphQLClient {
    /// Registers a profile at runtime (`registerPattern`) - the
    /// `ProfileFile`'s geometry is flattened into the InputObject's
    /// `[x0, y0, x1, y1, ...]` form. The result union carries the
    /// assigned pattern ID or the typed rejection.
    pub async fn register_pattern(
        &self,
        profile: &ProfileFile,
        owner: Option<&Owner>,
        lifetime: Lifetime,
    ) -> Result<RegisterPatternResult, ClientError> {
        let fragment: RegisterPatternResultFragment = self
            .call_mutation::<RegisterPattern, _, _>(
                "registerPattern",
                RegisterPatternVariables {
                    input: profile.into(),
                    owner: owner.cloned(),
                    lifetime: Some(lifetime),
                },
            )
            .await?;
        fragment.into_result()
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

    /// Claims an owner-less profile for `owner` (`claimProfile`) - the
    /// application adopts a statically loaded or released profile so
    /// `unregister_owner`/disconnect cleanup releases it back to owner-less instead of unregistering it. The profile is
    /// addressed by ID - `lookup_profile`'s match carries it; profile
    /// names are not unique. `enable` activates the profile in the
    /// same operation (cleanup reverts that activation). The result
    /// union carries the claimed profile or the typed rejection
    /// (`PATTERN_NOT_FOUND`, `PROFILE_ALREADY_OWNED`).
    pub async fn claim_profile(
        &self,
        id: PatternId,
        owner: &Owner,
        enable: bool,
    ) -> Result<ClaimProfileResult, ClientError> {
        let fragment: ClaimProfileResultFragment = self
            .call_mutation::<ClaimProfile, _, _>(
                "claimProfile",
                ClaimProfileVariables {
                    id,
                    owner: owner.clone(),
                    enable: Some(enable),
                },
            )
            .await?;
        fragment.into_result()
    }

    /// Releases `owner`'s claim on a profile (`releaseProfile`) - the
    /// profile stays registered and keeps matching; it only leaves the
    /// owner's cleanup scope. `disable` deactivates the profile in
    /// the same operation - the counterpart of `claim_profile`'s
    /// `enable`. The result union carries the released profile or the
    /// typed rejection (`PATTERN_NOT_FOUND`, `PROFILE_NOT_OWNED_BY`).
    pub async fn release_profile(
        &self,
        id: PatternId,
        owner: &Owner,
        disable: bool,
    ) -> Result<ReleaseProfileResult, ClientError> {
        let fragment: ReleaseProfileResultFragment = self
            .call_mutation::<ReleaseProfile, _, _>(
                "releaseProfile",
                ReleaseProfileVariables {
                    id,
                    owner: owner.clone(),
                    disable: Some(disable),
                },
            )
            .await?;
        fragment.into_result()
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
