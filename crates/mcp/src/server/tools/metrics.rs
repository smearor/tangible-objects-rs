//! Metrics tools - `get_metrics`, `reset_metrics`.

use rmcp::ErrorData;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::json_pretty;

/// Session-metrics tools.
#[tool_router(router = metrics_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Returns the accumulated session metrics.
    #[tool(
        description = "Get session recognition metrics: frame uptime, tangible re-ID churn, exact/partial match frames, claimed pins, per-slot tracking-ID churn."
    )]
    async fn get_metrics(&self) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&self.handle.snapshot().metrics)?,
        )]))
    }

    /// Resets the session metric counters.
    #[tool(
        description = "Reset the accumulated session metrics - call between automated test runs to isolate each series."
    )]
    async fn reset_metrics(&self) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::ResetMetrics)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(
            "metrics reset".to_string(),
        )]))
    }
}
