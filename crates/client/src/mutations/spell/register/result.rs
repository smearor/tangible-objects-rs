//! `RegisterSpellResult` - union decode for the `registerSpell`
//! mutation.

use cynic::InlineFragments;
use tangible_objects_model::RegisterSpellResult;
use tangible_objects_model::RegistrationRejected;
use tangible_objects_model::SpellRegistered;

use crate::ClientError;

/// `registerSpell` union selection - `__typename` picks the member.
/// The fragment types are the shared model shapes; `into_result`
/// folds the decode enum onto the domain union.
#[derive(InlineFragments)]
#[cynic(
    graphql_type = "RegisterSpellResult",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) enum RegisterSpellResultFragment {
    /// `SpellRegistered` member.
    SpellRegistered(SpellRegistered),
    /// `RegistrationRejected` member.
    RegistrationRejected(RegistrationRejected),
    /// Unknown future members - reported instead of silently lost.
    #[cynic(fallback)]
    Other,
}

impl RegisterSpellResultFragment {
    /// Folds the decode enum onto the domain union.
    pub(crate) fn into_result(self) -> Result<RegisterSpellResult, ClientError> {
        match self {
            Self::SpellRegistered(registered) => Ok(RegisterSpellResult::Registered(registered)),
            Self::RegistrationRejected(rejected) => Ok(RegisterSpellResult::Rejected(rejected)),
            Self::Other => Err(ClientError::Protocol(
                "unknown registerSpell result member".to_string(),
            )),
        }
    }
}
