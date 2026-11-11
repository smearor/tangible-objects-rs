//! `RemovedRegistrations` - what `Engine::remove_by_owner` tore
//! down, so the caller can emit lifecycle events and unpersist the
//! right store files.

use tangible_objects_model::DeadZoneName;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;
use tangible_objects_model::SpellName;

/// The registrations removed for one owner (`UnregisterOwner` /
/// D-Bus disconnect cleanup).
pub struct RemovedRegistrations {
    /// Removed patterns `(id, name)` - the name is needed to
    /// unpersist `profiles.d`.
    pub patterns: Vec<(PatternId, PatternName)>,
    /// Removed field sets by name.
    pub field_sets: Vec<FieldSetName>,
    /// Removed dead zones by name.
    pub dead_zones: Vec<DeadZoneName>,
    /// Removed spells by name.
    pub spells: Vec<SpellName>,
}
