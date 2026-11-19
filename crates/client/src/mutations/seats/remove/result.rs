//! `RemoveSeatResult` - union decode for the `removeSeat` mutation.

use cynic::InlineFragments;
use tangible_objects_model::RegistrationRejected;
use tangible_objects_model::RemoveSeatResult;
use tangible_objects_model::SeatRemoved;

use crate::ClientError;

/// `removeSeat` union selection - `__typename` picks the member. The
/// fragment types are the shared model shapes; `into_result` folds
/// the decode enum onto the domain union.
#[derive(InlineFragments)]
#[cynic(
    graphql_type = "RemoveSeatResult",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) enum RemoveSeatResultFragment {
    /// `SeatRemoved` member.
    SeatRemoved(SeatRemoved),
    /// `RegistrationRejected` member.
    RegistrationRejected(RegistrationRejected),
    /// Unknown future members - reported instead of silently lost.
    #[cynic(fallback)]
    Other,
}

impl RemoveSeatResultFragment {
    /// Folds the decode enum onto the domain union.
    pub(crate) fn into_result(self) -> Result<RemoveSeatResult, ClientError> {
        match self {
            Self::SeatRemoved(removed) => Ok(RemoveSeatResult::Removed(removed)),
            Self::RegistrationRejected(rejected) => Ok(RemoveSeatResult::Rejected(rejected)),
            Self::Other => Err(ClientError::Protocol(
                "unknown removeSeat result member".to_string(),
            )),
        }
    }
}
