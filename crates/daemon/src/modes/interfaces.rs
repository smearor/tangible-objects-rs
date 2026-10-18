//! Interface endpoint selection for the live modes.

use std::net::SocketAddr;

/// Optional interface endpoints for the live mode.
#[derive(Debug, Default, Clone)]
pub struct Interfaces {
    /// GraphQL listen address, if the HTTP/WebSocket API is enabled.
    pub graphql_addr: Option<SocketAddr>,
    /// Whether to serve the D-Bus session-bus interface.
    pub dbus: bool,
    /// MCP streamable-HTTP listen address, if enabled.
    pub mcp_addr: Option<SocketAddr>,
}
