//! Shared runtime: event distribution, state snapshots and the control
//! channel between the daemon's device loop and the interface crates.
//!
//! [`EngineRuntime`] lives in the daemon's synchronous device loop: every
//! processed [`tangible_objects_engine::EngineFrame`] is published once,
//! updating the snapshot and broadcasting domain events. Interface crates
//! (GraphQL, D-Bus, MCP) receive a cloneable [`EngineHandle`] with stream,
//! snapshot and control access - they never touch the engine directly.

mod command;
mod engine_handle;
mod engine_runtime;
mod snapshot;
mod training_state;

pub use command::ControlCommand;
pub use engine_handle::EngineHandle;
pub use engine_runtime::COMMAND_CHANNEL_CAPACITY;
pub use engine_runtime::EVENT_CHANNEL_CAPACITY;
pub use engine_runtime::EngineRuntime;
pub use snapshot::EngineSnapshot;
pub use tangible_objects_model::DeviceInfo;
pub use tangible_objects_model::DomainEvent;
pub use tangible_objects_model::EventKind;
pub use tangible_objects_model::LifecycleEvent;
pub use tangible_objects_model::PatternMetrics;
pub use tangible_objects_model::ProfileInfo;
pub use tangible_objects_model::RecordingStatus;
pub use tangible_objects_model::SessionMetrics;
pub use tangible_objects_model::StreamEvent;
pub use tangible_objects_model::TangibleEvent;
pub use training_state::TrainingState;
