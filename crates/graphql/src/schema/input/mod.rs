//! Mutation input objects - wire DTOs for `registerPattern` /
//! `registerFieldSet` plus their domain projections.

mod field_input;
mod register_field_set_input;
mod register_pattern_input;
mod window_input;

pub(crate) use register_field_set_input::RegisterFieldSetInput;
pub(crate) use register_pattern_input::RegisterPatternInput;
