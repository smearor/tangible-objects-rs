//! `ReleaseSeatSessionResult` - union decode for the
//! `releaseSeatSession` mutation.

use cynic::InlineFragments;
use tangible_objects_model::RegistrationRejected;
use tangible_objects_model::ReleaseSeatSessionResult;
use tangible_objects_model::SeatSessionReleased;

use crate::ClientError;

/// `releaseSeatSession` union selection - `__typename` picks the
/// member. The fragment types are the shared model shapes;
/// `into_result` folds the decode enum onto the domain union.
#[derive(InlineFragments)]
#[cynic(
    graphql_type = "ReleaseSeatSessionResult",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) enum ReleaseSeatSessionResultFragment {
    /// `SeatSessionReleased` member.
    SeatSessionReleased(SeatSessionReleased),
    /// `RegistrationRejected` member.
    RegistrationRejected(RegistrationRejected),
    /// Unknown future members - reported instead of silently lost.
    #[cynic(fallback)]
    Other,
}

impl ReleaseSeatSessionResultFragment {
    /// Folds the decode enum onto the domain union.
    pub(crate) fn into_result(self) -> Result<ReleaseSeatSessionResult, ClientError> {
        match self {
            Self::SeatSessionReleased(released) => Ok(ReleaseSeatSessionResult::Released(released)),
            Self::RegistrationRejected(rejected) => {
                Ok(ReleaseSeatSessionResult::Rejected(rejected))
            }
            Self::Other => Err(ClientError::Protocol(
                "unknown releaseSeatSession result member".to_string(),
            )),
        }
    }
}
