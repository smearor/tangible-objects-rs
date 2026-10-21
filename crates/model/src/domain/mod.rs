//! Domain event types - produced by the engine/runtime, consumed by the
//! interface crates and remote clients.
//!
//! The wire-facing types (`EventKind`, `StreamEvent`, `TangibleEvent`)
//! carry serde in the GraphQL schema's conventions (SCREAMING_SNAKE
//! enums, camelCase fields) so a GraphQL client can deserialize them
//! directly - same types, one home.

mod domain_event;
mod engine_event;
mod event_kind;
mod lifecycle_event;
mod stream_event;
mod tangible_event;

pub use domain_event::DomainEvent;
pub use engine_event::EngineEvent;
pub use event_kind::EventKind;
pub use lifecycle_event::LifecycleEvent;
pub use stream_event::StreamEvent;
pub use tangible_event::TangibleEvent;
