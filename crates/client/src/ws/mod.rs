//! `graphql-transport-ws` subscription transport - `connection`
//! adapts the `tokio-tungstenite` socket to `graphql-ws-client`,
//! `operation` decodes `next` frame payloads, `api` runs the
//! subscription.

pub(crate) mod api;
mod connection;
mod message;
mod operation;

pub(crate) use api::subscribe;
