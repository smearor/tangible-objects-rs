//! D-Bus interface for the tangible detection engine.
//!
//! Exposes the [`EngineRuntime`](tangible_objects_runtime::EngineRuntime)
//! state on the session bus: `org.tangibleobjects.Engine` serves
//! `org.tangibleobjects.Engine1` at `/org/tangibleobjects/Engine` with
//! methods for queries/control and signals for domain events.
//!
//! [`serve`] runs the service; [`spawn`] runs it on a dedicated thread with
//! its own tokio runtime so the synchronous device loop stays untouched.

mod profile;
mod serve;
mod service;
mod tangible;

pub use profile::Profile;
pub use serve::BUS_NAME;
pub use serve::OBJECT_PATH;
pub use serve::serve;
pub use serve::spawn;
pub use service::EngineService;
pub use tangible::Tangible;
