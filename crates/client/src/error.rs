//! Errors the [`crate::GraphQLClient`] can raise.

use thiserror::Error;

/// Failure modes of client calls - transport, protocol and GraphQL-level
/// errors stay distinguishable (GraphQL `errors` arrays keep their
/// structure, not a stringified blob).
#[derive(Debug, Error)]
pub enum ClientError {
    /// HTTP request failed (unreachable host, 4xx/5xx, body decode).
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    /// WebSocket handshake or frame handling failed - boxed: the
    /// tungstenite error enum is ~140 bytes and would dominate
    /// `size_of::<ClientError>`.
    #[error("WebSocket error: {0}")]
    Ws(Box<tokio_tungstenite::tungstenite::Error>),
    /// `graphql-transport-ws` subscription failure reported by
    /// `graphql-ws-client` - handshake, keepalive timeout, GraphQL
    /// `error` frames and frame decode all surface here.
    #[error("subscription transport error: {0}")]
    Subscription(#[from] graphql_ws_client::Error),
    /// Malformed handshake - no `connection_ack`, or a `next` message
    /// without the subscription's payload.
    #[error("graphql-transport-ws protocol violation: {0}")]
    Protocol(String),
    /// The endpoint returned a GraphQL `errors` array (parse/validation
    /// errors, resolver failures).
    #[error("GraphQL errors: {0}")]
    Graphql(serde_json::Value),
    /// A response decoded but carried no `data` payload (absent or
    /// `null`).
    #[error("response contained no data")]
    NoData,
    /// The `data` object did not contain the expected operation field
    /// - the server never ran the selection.
    #[error("response data has no field '{0}'")]
    FieldNotFound(String),
    /// A `data` field failed to deserialize into the expected type.
    #[error("response decode failed: {0}")]
    Decode(serde_json::Error),
}

impl From<tokio_tungstenite::tungstenite::Error> for ClientError {
    /// Wraps the transport error boxed (see [`ClientError::Ws`]).
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::Ws(Box::new(e))
    }
}
