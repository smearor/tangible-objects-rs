//! `ReleaseProfileResult` - union decode for the `releaseProfile`
//! mutation.

use cynic::InlineFragments;
use tangible_objects_model::ProfileReleased;
use tangible_objects_model::RegistrationRejected;
use tangible_objects_model::ReleaseProfileResult;

use crate::ClientError;

/// `releaseProfile` union selection - `__typename` picks the member.
/// The fragment types are the shared model shapes; `into_result` folds
/// the decode enum onto the domain union.
#[derive(InlineFragments)]
#[cynic(
    graphql_type = "ReleaseProfileResult",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) enum ReleaseProfileResultFragment {
    /// `ProfileReleased` member.
    ProfileReleased(ProfileReleased),
    /// `RegistrationRejected` member.
    RegistrationRejected(RegistrationRejected),
    /// Unknown future members - reported instead of silently lost.
    #[cynic(fallback)]
    Other,
}

impl ReleaseProfileResultFragment {
    /// Folds the decode enum onto the domain union.
    pub(crate) fn into_result(self) -> Result<ReleaseProfileResult, ClientError> {
        match self {
            Self::ProfileReleased(released) => Ok(ReleaseProfileResult::Released(released)),
            Self::RegistrationRejected(rejected) => Ok(ReleaseProfileResult::Rejected(rejected)),
            Self::Other => Err(ClientError::Protocol(
                "unknown releaseProfile result member".to_string(),
            )),
        }
    }
}
