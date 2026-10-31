//! `RegisterFieldSetResult` - union decode for the `registerFieldSet`
//! mutation.

use cynic::InlineFragments;
use tangible_objects_model::FieldSetRegistered;
use tangible_objects_model::RegisterFieldSetResult;
use tangible_objects_model::RegistrationRejected;

use crate::ClientError;

/// `registerFieldSet` union selection - `__typename` picks the member.
/// The fragment types are the shared model shapes; `into_result` folds
/// the decode enum onto the domain union.
#[derive(InlineFragments)]
#[cynic(
    graphql_type = "RegisterFieldSetResult",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) enum RegisterFieldSetResultFragment {
    /// `FieldSetRegistered` member.
    FieldSetRegistered(FieldSetRegistered),
    /// `RegistrationRejected` member.
    RegistrationRejected(RegistrationRejected),
    /// Unknown future members - reported instead of silently lost.
    #[cynic(fallback)]
    Other,
}

impl RegisterFieldSetResultFragment {
    /// Folds the decode enum onto the domain union.
    pub(crate) fn into_result(self) -> Result<RegisterFieldSetResult, ClientError> {
        match self {
            Self::FieldSetRegistered(registered) => {
                Ok(RegisterFieldSetResult::Registered(registered))
            }
            Self::RegistrationRejected(rejected) => Ok(RegisterFieldSetResult::Rejected(rejected)),
            Self::Other => Err(ClientError::Protocol(
                "unknown registerFieldSet result member".to_string(),
            )),
        }
    }
}
