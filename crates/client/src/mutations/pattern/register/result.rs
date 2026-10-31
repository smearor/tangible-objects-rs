//! `RegisterPatternResult` - union decode for the `registerPattern`
//! mutation.

use cynic::InlineFragments;
use tangible_objects_model::PatternRegistered;
use tangible_objects_model::RegisterPatternResult;
use tangible_objects_model::RegistrationRejected;

use crate::ClientError;

/// `registerPattern` union selection - `__typename` picks the member.
/// The fragment types are the shared model shapes; `into_result` folds
/// the decode enum onto the domain union.
#[derive(InlineFragments)]
#[cynic(
    graphql_type = "RegisterPatternResult",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) enum RegisterPatternResultFragment {
    /// `PatternRegistered` member.
    PatternRegistered(PatternRegistered),
    /// `RegistrationRejected` member.
    RegistrationRejected(RegistrationRejected),
    /// Unknown future members - reported instead of silently lost.
    #[cynic(fallback)]
    Other,
}

impl RegisterPatternResultFragment {
    /// Folds the decode enum onto the domain union.
    pub(crate) fn into_result(self) -> Result<RegisterPatternResult, ClientError> {
        match self {
            Self::PatternRegistered(registered) => {
                Ok(RegisterPatternResult::Registered(registered))
            }
            Self::RegistrationRejected(rejected) => Ok(RegisterPatternResult::Rejected(rejected)),
            Self::Other => Err(ClientError::Protocol(
                "unknown registerPattern result member".to_string(),
            )),
        }
    }
}
