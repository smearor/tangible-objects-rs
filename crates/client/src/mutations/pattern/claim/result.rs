//! `ClaimProfileResult` - union decode for the `claimProfile` mutation.

use cynic::InlineFragments;
use tangible_objects_model::ClaimProfileResult;
use tangible_objects_model::ProfileClaimed;
use tangible_objects_model::RegistrationRejected;

use crate::ClientError;

/// `claimProfile` union selection - `__typename` picks the member.
/// The fragment types are the shared model shapes; `into_result` folds
/// the decode enum onto the domain union.
#[derive(InlineFragments)]
#[cynic(
    graphql_type = "ClaimProfileResult",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) enum ClaimProfileResultFragment {
    /// `ProfileClaimed` member.
    ProfileClaimed(ProfileClaimed),
    /// `RegistrationRejected` member.
    RegistrationRejected(RegistrationRejected),
    /// Unknown future members - reported instead of silently lost.
    #[cynic(fallback)]
    Other,
}

impl ClaimProfileResultFragment {
    /// Folds the decode enum onto the domain union.
    pub(crate) fn into_result(self) -> Result<ClaimProfileResult, ClientError> {
        match self {
            Self::ProfileClaimed(claimed) => Ok(ClaimProfileResult::Claimed(claimed)),
            Self::RegistrationRejected(rejected) => Ok(ClaimProfileResult::Rejected(rejected)),
            Self::Other => Err(ClientError::Protocol(
                "unknown claimProfile result member".to_string(),
            )),
        }
    }
}
