//! MCP server exposing the tangible detection engine to AI clients.
//!
//! Wraps an [`EngineHandle`](tangible_objects_runtime::EngineHandle) as an
//! `rmcp` server: tools for queries and control (`list_tangibles`,
//! `get_pose`, `list_profiles`, `set_filter_mode`, `train_profile`) plus
//! resources under `tangible://` (tangibles, touch points, sets, profiles).
//!
//! [`serve_stdio`] speaks MCP on stdin/stdout; [`spawn_http`] serves a
//! streamable-HTTP endpoint on a dedicated thread so the daemon's
//! synchronous device loop stays untouched.

mod serve;
mod server;
mod tool_args;

pub use serve::serve_http;
pub use serve::serve_stdio;
pub use serve::spawn_http;
pub use server::EngineMcp;
