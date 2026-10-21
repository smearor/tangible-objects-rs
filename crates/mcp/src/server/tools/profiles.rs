//! Profile query tools - `list_profiles`.

use rmcp::ErrorData;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::json_pretty;

/// Loaded-profile query tools.
#[tool_router(router = profiles_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Loaded figure profiles.
    #[tool(description = "List the loaded figure profiles.")]
    async fn list_profiles(&self) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&self.handle.profiles())?,
        )]))
    }
}
