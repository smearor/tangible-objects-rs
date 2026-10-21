//! Owner tools - `unregister_owner`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_model::Owner;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::tool_args::UnregisterOwnerArgs;

/// Owner-scoped cleanup tools.
#[tool_router(router = owner_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Removes every registration of an owner (app cleanup).
    #[tool(description = "Remove every pattern and field set owned by the given owner.")]
    async fn unregister_owner(
        &self,
        Parameters(args): Parameters<UnregisterOwnerArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::UnregisterOwner {
                owner: Owner::new(args.owner.clone()),
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "removed all registrations of {}",
            args.owner
        ))]))
    }
}
