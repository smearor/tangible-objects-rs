//! GUI tools - `set_gui_visible`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::tool_args::SetGuiVisibleArgs;

/// Daemon-GUI visibility tools.
#[tool_router(router = gui_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Shows or hides the daemon-embedded GUI window (`--gui` mode).
    #[tool(
        description = "Show or hide the daemon-embedded GUI window (--gui mode); no-op for headless daemons."
    )]
    async fn set_gui_visible(
        &self,
        Parameters(args): Parameters<SetGuiVisibleArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::SetGuiVisible(args.visible))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "gui visible: {}",
            args.visible
        ))]))
    }
}
