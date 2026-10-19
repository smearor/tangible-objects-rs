//! Headless detection engine for tangible objects on multitouch surfaces.
//!
//! Feeds raw multitouch events (evdev or `evtest` recordings) through the
//! slot state machine, matches geometric figure patterns and tracks detected
//! tangibles across frames. Linux-only for device access; the detection core
//! itself is platform-agnostic.

mod convert;
mod device;
mod engine;
mod proxy;
mod recorder;
mod replay;

pub use convert::ConvertStats;
pub use convert::DeviceConverter;
pub use device::DeviceEntry;
pub use device::TouchDevice;
pub use engine::Engine;
pub use engine::EngineError;
pub use engine::EngineEvent;
pub use engine::EngineFrame;
pub use engine::RegistrationError;
pub use engine::TouchState;
pub use engine::Tracker;
pub use proxy::LegacyMode;
pub use proxy::Pipeline;
pub use proxy::ScopeChanges;
pub use proxy::StageMetrics;
pub use proxy::VirtualTouchscreen;
pub use recorder::FrameContext;
pub use recorder::FrameRecorder;
pub use recorder::RawRecorder;
pub use recorder::RecordingSession;
pub use replay::EvtestReplay;

pub use tangible_objects_model as model;
