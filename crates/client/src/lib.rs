//! Typed GraphQL client for the tangible-objects daemon
//! (GRAPHQL_CLIENT.md).
//!
//! [`GraphQLClient`] covers the full schema surface: all queries,
//! mutations and subscriptions, typed end to end - model types in,
//! model types out. Documents are built by `cynic` fragments checked
//! against `crates/graphql/schema.graphql` at compile time (no
//! hand-written query strings); subscriptions run over the
//! `graphql-transport-ws` protocol on a raw `tokio-tungstenite`
//! socket.
//!
//! # Quickstart
//!
//! ```no_run
//! use tangible_objects_client::Endpoint;
//! use tangible_objects_client::GraphQLClient;
//! use tangible_objects_client::ClientError;
//!
//! # async fn example() -> Result<(), ClientError> {
//! let endpoint = Endpoint::resolve(None); // flag/env > client.toml > default
//! let client = GraphQLClient::new(endpoint)?;
//! for tangible in client.tangibles().await? {
//!     println!("{} @ ({:.1},{:.1})", tangible.pattern_name.as_str(), tangible.center.x, tangible.center.y);
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod config;
mod endpoint;
mod error;
mod mutations;
mod queries;
mod subscriptions;
mod ws;

pub use client::GraphQLClient;
pub use config::ClientConfig;
pub use endpoint::Endpoint;
pub use error::ClientError;
pub use queries::profiles::api::ProfileQuery;
pub use subscriptions::events::api::EventStreamQuery;
/// Model types consumed and returned by the client - re-exported so
/// callers don't need a second `tangible-objects-model` dependency for
/// the common cases (`ProfileFile`, `FieldSet`, `FilterMode`, ...).
pub use tangible_objects_model as model;
