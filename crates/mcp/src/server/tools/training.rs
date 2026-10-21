//! Training tools - `train_profile`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::tool_args::TrainProfileArgs;

/// Training-session tools.
#[tool_router(router = training_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Starts a training session capturing a new profile.
    #[tool(description = "Start a training session capturing a new profile with the given name.")]
    async fn train_profile(
        &self,
        Parameters(args): Parameters<TrainProfileArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::TrainProfile(args.name.to_string()))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "training started for '{}'",
            args.name
        ))]))
    }
}
