//! MCP server surface - `engine_mcp` holds the type, `tools/` the
//! per-topic `#[tool_router]` impls, `handler` the `ServerHandler`
//! impl and `helpers` shared plumbing.

mod engine_mcp;
mod handler;
mod helpers;
mod tools;

pub use engine_mcp::EngineMcp;
