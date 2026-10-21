//! Device tools - `device_info`.

use rmcp::ErrorData;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::json_pretty;

/// Attached-device query tools.
#[tool_router(router = device_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Attached input device info (name + axis calibration).
    #[tool(
        description = "Get the attached touchscreen's name and axis calibration - needed to convert this host's raw recordings to another device."
    )]
    async fn device_info(&self) -> Result<CallToolResult, ErrorData> {
        let snapshot = self.handle.snapshot();
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&snapshot.device)?,
        )]))
    }
}
