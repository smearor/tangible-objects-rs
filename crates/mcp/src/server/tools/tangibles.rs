//! Tangible-state tools - `list_tangibles`, `get_pose`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::json_pretty;
use crate::tool_args::GetPoseArgs;

/// Tangible query tools.
#[tool_router(router = tangibles_router, vis = "pub(crate)")]
impl EngineMcp {
    /// All currently tracked tangibles with smoothed pose.
    #[tool(description = "List all tangibles currently tracked on the multitouch surface.")]
    async fn list_tangibles(&self) -> Result<CallToolResult, ErrorData> {
        let snapshot = self.handle.snapshot();
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&snapshot.tracked)?,
        )]))
    }

    /// Center and rotation of one tangible by tracking ID.
    #[tool(description = "Get center and rotation of a tangible by tracking ID.")]
    async fn get_pose(
        &self,
        Parameters(args): Parameters<GetPoseArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let snapshot = self.handle.snapshot();
        let Some(tangible) = snapshot.tracked.iter().find(|t| t.id == args.tangible_id) else {
            return Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "no tangible with id {}",
                args.tangible_id.as_u32()
            ))]));
        };
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(tangible)?,
        )]))
    }
}
