//! Detection pipeline: slot state machine, pattern matching and tracking.
//!
//! [`TouchState`] turns raw events into contacts, `matching` finds figure
//! patterns among them (private), [`Tracker`] smooths poses across frames and
//! [`Engine`] orchestrates everything, emitting an [`EngineFrame`] per
//! `SYN_REPORT` that carries domain [`EngineEvent`]s (touch changes, tangible
//! added/moved/removed) for interface crates.

mod best_match;
mod detection;
mod diagnostics;
mod error;
mod event;
mod fields;
mod frame;
mod matching;
mod touch_state;
mod tracker;

pub use detection::Engine;
pub use error::EngineError;
pub use event::EngineEvent;
pub use fields::FieldTracker;
pub use frame::EngineFrame;
pub use touch_state::TouchState;
pub use tracker::Tracker;
