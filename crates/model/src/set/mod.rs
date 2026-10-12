//! Tangible Object Sets (TOS): grouping of tangibles with activation
//! hysteresis.
//!
//! A [`TangibleSet`] defines which patterns belong to a set (mandatory and
//! additional members). [`SetTracker`] derives [`SetAction`]s — the TOS
//! action vocabulary from `TANGIBLE_OBJECT_SET.md` — from the engine's
//! tracked tangibles.

mod action;
mod definition;
mod id;
mod state;
mod tracker;

pub use action::SetAction;
pub use definition::TangibleSet;
pub use id::TangibleSetId;
pub use state::TangibleSetState;
pub use tracker::SetTracker;
