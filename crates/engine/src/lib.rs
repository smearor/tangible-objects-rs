//! Headless detection engine for tangible objects on multitouch surfaces.
//!
//! Feeds raw multitouch events (evdev or `evtest` recordings) through the
//! slot state machine, matches geometric figure patterns and tracks detected
//! tangibles across frames. Linux-only for device access; the detection core
//! itself is platform-agnostic.

mod device;
mod engine;
mod engine_error;
mod engine_frame;
mod matching;
mod replay;
mod touch_state;
mod tracker;

pub use device::DeviceEntry;
pub use device::TouchDevice;
pub use engine::Engine;
pub use engine_error::EngineError;
pub use engine_frame::EngineFrame;
pub use replay::EvtestReplay;
pub use touch_state::TouchState;
pub use tracker::Tracker;

pub use tangible_objects_model as model;
