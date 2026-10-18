use crate::FieldSet;
use crate::ProfileFile;

/// A dynamically registerable entity (DYNAMIC_PROFILES.md).
///
/// Applications upload their tangible definitions at session start via
/// the interfaces; registration rejects fingerprints colliding with an
/// already-registered profile.
#[derive(Debug, Clone)]
pub enum Registration {
    /// A figure profile (pattern) - registered as a [`TangiblePattern`]
    /// with a fresh monotonic [`PatternId`].
    Pattern(ProfileFile),
    /// An occupancy field set.
    FieldSet(FieldSet),
}
