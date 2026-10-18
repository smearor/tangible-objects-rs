//! GraphQL interface for the tangible detection engine.
//!
//! Exposes the [`EngineRuntime`](tangible_objects_runtime::EngineRuntime)
//! state through `async-graphql`: queries read the latest snapshot and
//! static configuration, mutations push [`ControlCommand`]s back to the
//! device loop and the `events` subscription streams
//! [`DomainEvent`]s as `TangibleEvent`s over WebSocket.
//!
//! [`serve`] binds the endpoint on a given address; [`spawn`] runs it on a
//! dedicated thread with its own tokio runtime so the synchronous device
//! loop stays untouched.

mod api;
mod schema;
mod serve;

pub use api::EventKind;
pub use api::StreamEvent;
pub use api::TangibleEvent;
pub use schema::EngineSchema;
pub use schema::MutationRoot;
pub use schema::QueryRoot;
pub use schema::SubscriptionRoot;
pub use serve::build_schema;
pub use serve::serve;
pub use serve::spawn;
