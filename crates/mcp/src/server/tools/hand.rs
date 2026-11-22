//! Hand tools - `register_hand_profile`, `unregister_hand_profile`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_model::HandProfile;
use tangible_objects_model::Registration;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::parse_lifetime;
use crate::tool_args::RegisterHandProfileArgs;
use crate::tool_args::UnregisterHandProfileArgs;

/// Hand registration tools (SPELLS.md's per-user spells).
#[tool_router(router = hand_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Registers a handprint from TOML.
    #[tool(
        description = "Register a five-contact handprint from TOML (name, tolerance, points, SPELLS.md's per-user spells - a `five-finger-tap` arming can bind spells to it)."
    )]
    async fn register_hand_profile(
        &self,
        Parameters(args): Parameters<RegisterHandProfileArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let hand = toml::from_str::<HandProfile>(&args.toml).map_err(|e| {
            ErrorData::invalid_params(format!("invalid hand profile TOML: {e}"), None)
        })?;
        let (command, rx) = ControlCommand::register(
            Registration::HandProfile(hand),
            args.owner,
            parse_lifetime(&args.lifetime)?,
        );
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(Ok(_)) => Ok(CallToolResult::success(vec![ContentBlock::text(
                "hand profile registered".to_string(),
            )])),
            Ok(Err(reason)) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "rejected: {reason}"
            ))])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the registration reply".to_string(),
            )])),
        }
    }

    /// Removes a registered handprint by name.
    #[tool(description = "Unregister a hand profile by name.")]
    async fn unregister_hand_profile(
        &self,
        Parameters(args): Parameters<UnregisterHandProfileArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::UnregisterHandProfile {
                name: args.name.clone(),
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "unregistered hand profile {}",
            args.name
        ))]))
    }
}
