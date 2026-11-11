//! Spell tools - `register_spell`, `unregister_spell`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_model::Registration;
use tangible_objects_model::Spell;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::parse_lifetime;
use crate::tool_args::RegisterSpellArgs;
use crate::tool_args::UnregisterSpellArgs;

/// Spell registration tools.
#[tool_router(router = spell_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Registers a spell gesture from TOML.
    #[tool(
        description = "Register a spell gesture from TOML (name, fingers, corridor, [[strokes]] per-finger template polylines, [arming] trigger, SPELLS.md)."
    )]
    async fn register_spell(
        &self,
        Parameters(args): Parameters<RegisterSpellArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let spell = toml::from_str::<Spell>(&args.toml)
            .map_err(|e| ErrorData::invalid_params(format!("invalid spell TOML: {e}"), None))?;
        let (command, rx) = ControlCommand::register(
            Registration::Spell(spell),
            args.owner,
            parse_lifetime(&args.lifetime)?,
        );
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(Ok(_)) => Ok(CallToolResult::success(vec![ContentBlock::text(
                "spell registered".to_string(),
            )])),
            Ok(Err(reason)) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "rejected: {reason}"
            ))])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the registration reply".to_string(),
            )])),
        }
    }

    /// Removes a registered spell by name.
    #[tool(description = "Unregister a spell by name.")]
    async fn unregister_spell(
        &self,
        Parameters(args): Parameters<UnregisterSpellArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::UnregisterSpell {
                name: args.name.clone(),
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "unregistered spell {}",
            args.name
        ))]))
    }
}
