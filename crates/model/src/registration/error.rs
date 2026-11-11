use crate::DeadZoneName;
use crate::FieldName;
use crate::FieldSetName;
use crate::PatternId;
use crate::PatternName;
use crate::Region;
use crate::SpellName;
use thiserror::Error;

/// Errors produced by runtime profile and field-set registration
/// (DYNAMIC_PROFILES.md).
///
/// Lives in `model` so the control-command reply channel can carry the
/// typed rejection to interface crates (GraphQL/D-Bus/MCP) instead of a
/// pre-formatted string.
#[derive(Debug, Error)]
pub enum RegistrationError {
    /// The referenced pattern ID is not registered.
    #[error("pattern {0} not found")]
    PatternNotFound(PatternId),
    /// The profile's geometric fingerprint collides with an already
    /// registered pattern - registration is rejected so the new
    /// profile cannot shadow the incumbent.
    #[error("fingerprint collides with registered profile '{0}'")]
    FingerprintCollision(PatternName),
    /// The field set contains overlapping fields - an occupancy field
    /// must not suppress the same slot twice. Carries the offending pair
    /// and the intersection region so the rejection explains itself in
    /// logs and responses.
    #[error("field set '{0}' has overlapping fields: '{1}' overlaps '{2}' (overlap: {3})")]
    FieldsOverlap(FieldSetName, FieldName, FieldName, Region),
    /// A field set with this name is already registered.
    #[error("field set '{0}' already registered")]
    DuplicateFieldSet(FieldSetName),
    /// A dead zone with this name is already registered.
    #[error("dead zone '{0}' already registered")]
    DuplicateDeadZone(DeadZoneName),
    /// The spell's exclusive zone overlaps a field - the zone
    /// reserves its area for gestures while the field expects
    /// occupancy (SPELLS.md). Carries the spell and the offending
    /// field so the rejection explains itself in logs and
    /// responses.
    #[error("spell '{0}' exclusive zone overlaps field '{1}'")]
    SpellZoneOverlapsField(SpellName, FieldName),
}
