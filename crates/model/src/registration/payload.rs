use crate::DeadZone;
use crate::FieldSet;
use crate::HandProfile;
use crate::ProfileFile;
use crate::Spell;

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
    /// A spell - a trainable multi-finger gesture (SPELLS.md).
    Spell(Spell),
    /// A dead zone - a blacklisted surface position suppressing ghost
    /// contacts at ingest.
    DeadZone(DeadZone),
    /// A handprint - the five-contact constellation `five-finger-tap`
    /// arming binds spells to (per-user spells, SPELLS.md).
    HandProfile(HandProfile),
}
