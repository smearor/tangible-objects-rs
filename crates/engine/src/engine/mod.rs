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
mod pin_map;
mod pin_maps;
mod registration_error;
mod smoothed_contact;
mod smoothed_contacts;
mod touch_state;
mod tracker;

pub(crate) use cell_state::CellState;
pub use detection::Engine;
pub use error::EngineError;
pub use field_tracker::FieldTracker;
pub use frame::EngineFrame;
pub use registration_error::RegistrationError;
pub use tangible_objects_model::EngineEvent;
pub use touch_state::TouchState;
pub use tracker::Tracker;
