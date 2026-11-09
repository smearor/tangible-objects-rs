//! Occupancy field sets for anonymous single-contact tangibles.
//!
//! A [`FieldSet`] declares a grid of named [`Field`]s (regions with
//! occupancy semantics) - either screen-absolute or relative to an
//! optional [`WindowAnchor`] (SINGLE_CONTACT_FIELD.md).

mod anchor;
mod cell;
mod count;
mod field_name;
mod name;
mod set;
mod set_count;

pub use anchor::WindowAnchor;
pub use cell::Field;
pub use count::FieldCount;
pub use field_name::FieldName;
pub use name::FieldSetName;
pub use set::FieldSet;
pub use set::default_stability;
pub use set_count::FieldSetCount;
