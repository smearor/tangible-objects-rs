//! Filter tools - `set_filter_mode`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_model::FilterMode;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::tool_args::SetFilterModeArgs;

/// Suppression-policy tools.
#[tool_router(router = filter_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Switches the filter proxy's suppression policy.
    #[tool(
        description = "Set the filter proxy mode: off, immediate, gated, guarded or tangibles-only."
    )]
    async fn set_filter_mode(
        &self,
        Parameters(args): Parameters<SetFilterModeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let mode = args.mode.parse::<FilterMode>().map_err(|_| {
            ErrorData::invalid_params(
                "mode must be one of: off, immediate, gated, guarded, tangibles-only",
                None,
            )
        })?;
        let stability = args.stability;
        let stage = args.stage;
        self.handle
            .send_command(ControlCommand::SetFilterMode {
                mode,
                stability,
                stage,
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "filter mode set to {mode}"
        ))]))
    }
}
