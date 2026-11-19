//! `UpdateSeatResult` - union decode for the `updateSeat` mutation.

use cynic::InlineFragments;
use tangible_objects_model::RegistrationRejected;
use tangible_objects_model::SeatUpdated;
use tangible_objects_model::UpdateSeatResult;

use crate::ClientError;

/// `updateSeat` union selection - `__typename` picks the member. The
/// fragment types are the shared model shapes; `into_result` folds
/// the decode enum onto the domain union.
#[derive(InlineFragments)]
#[cynic(
    graphql_type = "UpdateSeatResult",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) enum UpdateSeatResultFragment {
    /// `SeatUpdated` member.
    SeatUpdated(SeatUpdated),
    /// `RegistrationRejected` member.
    RegistrationRejected(RegistrationRejected),
    /// Unknown future members - reported instead of silently lost.
    #[cynic(fallback)]
    Other,
}

impl UpdateSeatResultFragment {
    /// Folds the decode enum onto the domain union.
    pub(crate) fn into_result(self) -> Result<UpdateSeatResult, ClientError> {
        match self {
            Self::SeatUpdated(updated) => Ok(UpdateSeatResult::Updated(updated)),
            Self::RegistrationRejected(rejected) => Ok(UpdateSeatResult::Rejected(rejected)),
            Self::Other => Err(ClientError::Protocol(
                "unknown updateSeat result member".to_string(),
            )),
        }
    }
}
