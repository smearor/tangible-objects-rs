//! `EngineMcp` - the MCP server type bridging the
//! [`EngineHandle`] to tools and resources. The tool impls live in
//! `crate::server::tools` topic modules; each contributes a named
//! `#[tool_router]` merged in `handler.rs`.

use tangible_objects_runtime::EngineHandle;

/// MCP server bridging the [`EngineHandle`] to tools and resources.
#[derive(Clone)]
pub struct EngineMcp {
    /// Runtime handle shared by all topic tool impls.
    pub(crate) handle: EngineHandle,
}

impl EngineMcp {
    /// Creates the MCP server for the given runtime handle.
    pub fn new(handle: EngineHandle) -> Self {
        Self { handle }
    }
}
