//! `RegisterDeadZoneResult` - union decode for the
//! `registerDeadZone` mutation.

use cynic::InlineFragments;
use tangible_objects_model::DeadZoneRegistered;
use tangible_objects_model::RegisterDeadZoneResult;
use tangible_objects_model::RegistrationRejected;

use crate::ClientError;

/// `registerDeadZone` union selection - `__typename` picks the
/// member. The fragment types are the shared model shapes;
/// `into_result` folds the decode enum onto the domain union.
#[derive(InlineFragments)]
#[cynic(
    graphql_type = "RegisterDeadZoneResult",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) enum RegisterDeadZoneResultFragment {
    /// `DeadZoneRegistered` member.
    DeadZoneRegistered(DeadZoneRegistered),
    /// `RegistrationRejected` member.
    RegistrationRejected(RegistrationRejected),
    /// Unknown future members - reported instead of silently lost.
    #[cynic(fallback)]
    Other,
}

impl RegisterDeadZoneResultFragment {
    /// Folds the decode enum onto the domain union.
    pub(crate) fn into_result(self) -> Result<RegisterDeadZoneResult, ClientError> {
        match self {
            Self::DeadZoneRegistered(registered) => {
                Ok(RegisterDeadZoneResult::Registered(registered))
            }
            Self::RegistrationRejected(rejected) => Ok(RegisterDeadZoneResult::Rejected(rejected)),
            Self::Other => Err(ClientError::Protocol(
                "unknown registerDeadZone result member".to_string(),
            )),
        }
    }
}
