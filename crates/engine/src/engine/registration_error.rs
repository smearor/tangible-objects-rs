use tangible_objects_model::FieldName;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;

/// Errors produced by runtime profile and field-set registration
/// (DYNAMIC_PROFILES.md).
#[derive(Debug, thiserror::Error)]
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
    /// so the rejection explains itself in logs and responses.
    #[error("field set '{0}' has overlapping fields: '{1}' overlaps '{2}'")]
    FieldsOverlap(FieldSetName, FieldName, FieldName),
    /// A field set with this name is already registered.
    #[error("field set '{0}' already registered")]
    DuplicateFieldSet(FieldSetName),
}
