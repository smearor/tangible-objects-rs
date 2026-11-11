//! Detection pipeline: slot state machine, pattern matching and tracking.
//!
//! [`TouchState`] turns raw events into contacts, `matching` finds figure
//! patterns among them (private), [`Tracker`] smooths poses across frames and
//! [`Engine`] orchestrates everything, emitting an [`EngineFrame`] per
//! `SYN_REPORT` that carries domain [`EngineEvent`]s (touch changes, tangible
//! added/moved/removed) for interface crates.

mod cell_state;
mod detection;
mod error;
pub(crate) mod field_tracker;
mod frame;
mod gesture_tracker;
mod pin_map;
mod pin_maps;
mod removed_registrations;
mod smoothed_contact;
mod smoothed_contacts;
mod spell_training;
mod touch_state;
mod tracker;
mod trajectory;

pub(crate) use cell_state::CellState;
pub use detection::Engine;
pub use error::EngineError;
pub use field_tracker::FieldTracker;
pub use frame::EngineFrame;
pub use gesture_tracker::GestureTracker;
pub use removed_registrations::RemovedRegistrations;
pub use spell_training::SPELL_TRAINING_REPETITIONS;
pub use tangible_objects_model::EngineEvent;
pub use tangible_objects_model::RegistrationError;
pub use touch_state::TouchState;
pub use tracker::Tracker;
pub use trajectory::Trajectory;
