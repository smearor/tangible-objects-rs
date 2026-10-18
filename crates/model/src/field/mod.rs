//! Occupancy field sets for anonymous single-contact tangibles.
//!
//! A [`FieldSet`] declares a grid of named [`Field`]s (regions with
//! occupancy semantics) - either screen-absolute or relative to an
//! optional [`WindowAnchor`] (SINGLE_CONTACT_FIELD.md).

mod anchor;
mod cell;
mod field_name;
mod name;
mod set;

pub use anchor::WindowAnchor;
pub use cell::Field;
pub use field_name::FieldName;
pub use name::FieldSetName;
pub use set::FieldSet;
pub use set::default_stability;
