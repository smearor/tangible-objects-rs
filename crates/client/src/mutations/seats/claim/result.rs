//! `ClaimSeatSessionResult` - union decode for the `claimSeatSession`
//! mutation.

use cynic::InlineFragments;
use tangible_objects_model::ClaimSeatSessionResult;
use tangible_objects_model::RegistrationRejected;
use tangible_objects_model::SeatSessionClaimed;

use crate::ClientError;

/// `claimSeatSession` union selection - `__typename` picks the
/// member. The fragment types are the shared model shapes;
/// `into_result` folds the decode enum onto the domain union.
#[derive(InlineFragments)]
#[cynic(
    graphql_type = "ClaimSeatSessionResult",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) enum ClaimSeatSessionResultFragment {
    /// `SeatSessionClaimed` member.
    SeatSessionClaimed(SeatSessionClaimed),
    /// `RegistrationRejected` member.
    RegistrationRejected(RegistrationRejected),
    /// Unknown future members - reported instead of silently lost.
    #[cynic(fallback)]
    Other,
}

impl ClaimSeatSessionResultFragment {
    /// Folds the decode enum onto the domain union.
    pub(crate) fn into_result(self) -> Result<ClaimSeatSessionResult, ClientError> {
        match self {
            Self::SeatSessionClaimed(claimed) => Ok(ClaimSeatSessionResult::Claimed(claimed)),
            Self::RegistrationRejected(rejected) => Ok(ClaimSeatSessionResult::Rejected(rejected)),
            Self::Other => Err(ClientError::Protocol(
                "unknown claimSeatSession result member".to_string(),
            )),
        }
    }
}
