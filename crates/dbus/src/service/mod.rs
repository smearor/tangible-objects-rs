//! D-Bus service - `engine_service` holds the object type,
//! `interface` the `#[interface]` wire surface delegating into the
//! `api/` topic modules, `signals` the domain-event mapping.

mod api;
mod engine_service;
mod interface;
pub(crate) mod signals;

pub use engine_service::EngineService;
pub use engine_service::OwnerSessions;
