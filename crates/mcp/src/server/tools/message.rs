//! Message tools - `show_message`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::tool_args::ShowMessageArgs;

/// GUI-overlay message tools.
#[tool_router(router = message_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Pushes a user-facing message into the GUI overlay.
    #[tool(
        description = "Show a user-facing message in the GUI overlay (e.g. 'Legen Sie Figur 10 auf'); an empty text clears it."
    )]
    async fn show_message(
        &self,
        Parameters(args): Parameters<ShowMessageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::ShowMessage(args.text))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(
            "message shown".to_string(),
        )]))
    }
}
