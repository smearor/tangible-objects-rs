//! Shared runtime: event distribution, state snapshots and the control
//! channel between the daemon's device loop and the interface crates.
//!
//! [`EngineRuntime`] lives in the daemon's synchronous device loop: every
//! processed [`tangible_objects_engine::EngineFrame`] is published once,
//! updating the snapshot and broadcasting domain events. Interface crates
//! (GraphQL, D-Bus, MCP) receive a cloneable [`EngineHandle`] with stream,
//! snapshot and control access — they never touch the engine directly.

mod command;
mod domain_event;
mod profile_info;
mod runtime;
mod snapshot;

pub use command::ControlCommand;
pub use domain_event::DomainEvent;
pub use profile_info::ProfileInfo;
pub use runtime::COMMAND_CHANNEL_CAPACITY;
pub use runtime::EVENT_CHANNEL_CAPACITY;
pub use runtime::EngineHandle;
pub use runtime::EngineRuntime;
pub use snapshot::EngineSnapshot;
