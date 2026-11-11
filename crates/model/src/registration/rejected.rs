use crate::FieldOverlap;
use crate::RegistrationError;
use crate::RejectionKind;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// A rejected registration - `reason` is the human-readable error
/// string, `kind` the machine category; structured payloads ride per
/// kind (`overlap` for `FIELDS_OVERLAP`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "RegistrationRejected",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct RegistrationRejected {
    /// Human-readable rejection reason (the `RegistrationError`
    /// message).
    pub reason: String,
    /// Machine-readable rejection category.
    pub kind: RejectionKind,
    /// Field-overlap detail - `Some` iff `kind == FIELDS_OVERLAP`.
    pub overlap: Option<FieldOverlap>,
}

impl From<RegistrationError> for RegistrationRejected {
    fn from(error: RegistrationError) -> Self {
        let (kind, overlap) = match &error {
            RegistrationError::PatternNotFound(_) => (RejectionKind::PatternNotFound, None),
            RegistrationError::FingerprintCollision(_) => {
                (RejectionKind::FingerprintCollision, None)
            }
            RegistrationError::FieldsOverlap(_, first, second, region) => (
                RejectionKind::FieldsOverlap,
                Some(FieldOverlap {
                    first: first.clone(),
                    second: second.clone(),
                    region: *region,
                }),
            ),
            RegistrationError::DuplicateFieldSet(_) => (RejectionKind::DuplicateFieldSet, None),
            RegistrationError::DuplicateDeadZone(_) => (RejectionKind::DuplicateDeadZone, None),
            RegistrationError::SpellZoneOverlapsField(_, _) => {
                (RejectionKind::SpellZoneOverlapsField, None)
            }
        };
        Self {
            reason: error.to_string(),
            kind,
            overlap,
        }
    }
}
