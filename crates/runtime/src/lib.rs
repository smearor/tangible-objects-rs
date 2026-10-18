//! Shared runtime: event distribution, state snapshots and the control
//! channel between the daemon's device loop and the interface crates.
//!
//! [`EngineRuntime`] lives in the daemon's synchronous device loop: every
//! processed [`tangible_objects_engine::EngineFrame`] is published once,
//! updating the snapshot and broadcasting domain events. Interface crates
//! (GraphQL, D-Bus, MCP) receive a cloneable [`EngineHandle`] with stream,
//! snapshot and control access - they never touch the engine directly.

mod command;
mod device_info;
mod domain_event;
mod engine_handle;
mod engine_runtime;
mod profile_info;
mod recording_status;
mod snapshot;

pub use command::ControlCommand;
pub use device_info::DeviceInfo;
pub use domain_event::DomainEvent;
pub use engine_handle::EngineHandle;
pub use engine_runtime::COMMAND_CHANNEL_CAPACITY;
pub use engine_runtime::EVENT_CHANNEL_CAPACITY;
pub use engine_runtime::EngineRuntime;
pub use profile_info::ProfileInfo;
pub use recording_status::RecordingStatus;
pub use snapshot::EngineSnapshot;
pub use tangible_objects_model::PatternMetrics;
pub use tangible_objects_model::SessionMetrics;
