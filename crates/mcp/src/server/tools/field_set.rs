//! Field-set tools - `register_field_set`, `unregister_field_set`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_model::FieldSet;
use tangible_objects_model::Registration;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::parse_lifetime;
use crate::tool_args::RegisterFieldSetArgs;
use crate::tool_args::UnregisterFieldSetArgs;

/// Occupancy field-set registration tools.
#[tool_router(router = field_set_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Registers an occupancy field set from TOML.
    #[tool(
        description = "Register an occupancy field set from TOML (name, optional stability/[window], [[fields]] cells)."
    )]
    async fn register_field_set(
        &self,
        Parameters(args): Parameters<RegisterFieldSetArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let set = toml::from_str::<FieldSet>(&args.toml)
            .map_err(|e| ErrorData::invalid_params(format!("invalid field set TOML: {e}"), None))?;
        let (command, rx) = ControlCommand::register(
            Registration::FieldSet(set),
            args.owner,
            parse_lifetime(&args.lifetime)?,
        );
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(Ok(())) => Ok(CallToolResult::success(vec![ContentBlock::text(
                "field set registered".to_string(),
            )])),
            Ok(Err(reason)) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "rejected: {reason}"
            ))])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the registration reply".to_string(),
            )])),
        }
    }

    /// Removes a field set by name.
    #[tool(description = "Unregister a field set by name.")]
    async fn unregister_field_set(
        &self,
        Parameters(args): Parameters<UnregisterFieldSetArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::UnregisterFieldSet {
                name: args.name.clone(),
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "unregistered field set {}",
            args.name
        ))]))
    }
}
