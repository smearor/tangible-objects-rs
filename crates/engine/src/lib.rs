//! Headless detection engine for tangible objects on multitouch surfaces.
//!
//! Feeds raw multitouch events (evdev or `evtest` recordings) through the
//! slot state machine, matches geometric figure patterns and tracks detected
//! tangibles across frames. Linux-only for device access; the detection core
//! itself is platform-agnostic.

mod device;
mod engine;
mod replay;

pub use device::DeviceEntry;
pub use device::TouchDevice;
pub use engine::Engine;
pub use engine::EngineError;
pub use engine::EngineEvent;
pub use engine::EngineFrame;
pub use engine::TouchState;
pub use engine::Tracker;
pub use replay::EvtestReplay;

pub use tangible_objects_model as model;
