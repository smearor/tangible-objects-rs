use super::TangibleSetId;
use crate::TrackedTangible;

/// Action on a Tangible Object Set, derived from tracked tangible changes.
///
/// Mirrors the TOS action vocabulary (`TOS-ADD`, `TOS-REM`, `TOS-OBJ-ADD`,
/// `TOS-OBJ-REM`, `TOS-OBJ-MOV`) from `TANGIBLE_OBJECT_SET.md`. Emitted by
/// [`crate::SetTracker::update`].
#[derive(Debug, Clone, PartialEq)]
pub enum SetAction {
    /// All mandatory members became present: the set is active (TOS-ADD).
    SetAdded(TangibleSetId),
    /// The last mandatory member disappeared: the set is inactive (TOS-REM).
    SetRemoved(TangibleSetId),
    /// A member tangible appeared while the set was active (TOS-OBJ-ADD).
    ObjectAdded(TangibleSetId, TrackedTangible),
    /// A member tangible disappeared (TOS-OBJ-REM). Also emitted for every
    /// member present at the moment the set deactivates.
    ObjectRemoved(TangibleSetId, TrackedTangible),
    /// A member tangible's pose changed while the set is active
    /// (TOS-OBJ-MOV).
    ObjectMoved(TangibleSetId, TrackedTangible),
}
