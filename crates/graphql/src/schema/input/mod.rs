//! Mutation input objects - wire DTOs for `registerPattern` /
//! `registerFieldSet` / `registerDeadZone` / `registerSpell` plus
//! their domain projections.

mod field_input;
mod region_input;
mod register_dead_zone_input;
mod register_field_set_input;
mod register_pattern_input;
mod register_spell_input;
mod spell_arming_input;
mod window_input;

pub(crate) use register_dead_zone_input::RegisterDeadZoneInput;
pub(crate) use register_field_set_input::RegisterFieldSetInput;
pub(crate) use register_pattern_input::RegisterPatternInput;
pub(crate) use register_spell_input::RegisterSpellInput;
