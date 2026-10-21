//! Stage tools - `register_stage`, `unregister_stage`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::tool_args::StageArgs;

/// Pipeline-stage registration tools.
#[tool_router(router = stage_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Claims or creates a pipeline stage for the given scope.
    #[tool(
        description = "Register a pipeline stage - persists until unregistered, so its filter mode can be set before any scoped pattern exists."
    )]
    async fn register_stage(
        &self,
        Parameters(args): Parameters<StageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::RegisterStage {
                name: args.name.clone(),
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "stage {} registered",
            args.name
        ))]))
    }

    /// Removes a pipeline stage - its suppression state dies with it.
    #[tool(description = "Unregister a pipeline stage by scope name.")]
    async fn unregister_stage(
        &self,
        Parameters(args): Parameters<StageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::UnregisterStage {
                name: args.name.clone(),
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "stage {} unregistered",
            args.name
        ))]))
    }
}
