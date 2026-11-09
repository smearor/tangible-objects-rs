//! `FieldSetRow` - one row of the field-sets card.

use tangible_objects_model::FieldSetName;
use tangible_objects_model::Owner;

/// Display-relevant slice of a registered
/// [`tangible_objects_model::FieldSet`] plus its registration owner -
/// built per frame in `evdev_thread` from `engine.field_sets()` and
/// `engine.field_set_owner()`.
#[derive(Debug, Clone)]
pub struct FieldSetRow {
    /// The set's name.
    pub name: FieldSetName,
    /// Registering application; `None` for statically loaded sets.
    pub owner: Option<Owner>,
    /// Whether the set currently tracks occupancy
    /// (`SetFieldSetEnabled` command).
    pub enabled: bool,
    /// Number of occupancy fields in the set.
    pub field_count: usize,
}
