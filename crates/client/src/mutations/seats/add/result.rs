//! `AddSeatResult` - union decode for the `addSeat` mutation.

use cynic::InlineFragments;
use tangible_objects_model::AddSeatResult;
use tangible_objects_model::RegistrationRejected;
use tangible_objects_model::SeatAdded;

use crate::ClientError;

/// `addSeat` union selection - `__typename` picks the member. The
/// fragment types are the shared model shapes; `into_result` folds
/// the decode enum onto the domain union.
#[derive(InlineFragments)]
#[cynic(
    graphql_type = "AddSeatResult",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) enum AddSeatResultFragment {
    /// `SeatAdded` member.
    SeatAdded(SeatAdded),
    /// `RegistrationRejected` member.
    RegistrationRejected(RegistrationRejected),
    /// Unknown future members - reported instead of silently lost.
    #[cynic(fallback)]
    Other,
}

impl AddSeatResultFragment {
    /// Folds the decode enum onto the domain union.
    pub(crate) fn into_result(self) -> Result<AddSeatResult, ClientError> {
        match self {
            Self::SeatAdded(added) => Ok(AddSeatResult::Added(added)),
            Self::RegistrationRejected(rejected) => Ok(AddSeatResult::Rejected(rejected)),
            Self::Other => Err(ClientError::Protocol(
                "unknown addSeat result member".to_string(),
            )),
        }
    }
}
