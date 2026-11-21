//! Training tools - `train_profile`, `train_spell`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_model::SpellArming;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::tool_args::TrainProfileArgs;
use crate::tool_args::TrainSpellArgs;

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
            .send_command(ControlCommand::TrainProfile(args.name.clone()))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "training started for '{}'",
            args.name
        ))]))
    }

    /// Starts a spell training session.
    #[tool(
        description = "Start a spell training session: the next 3 completed gesture episodes become the trained spell's template strokes and corridor, then the spell registers persistently (SPELLS.md's learn-by-repetition). `arming` takes the [arming] table as TOML (e.g. `type = \"five-finger-tap\"`); omit for `free`."
    )]
    async fn train_spell(
        &self,
        Parameters(args): Parameters<TrainSpellArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let arming = match args.arming.as_deref() {
            None | Some("") => SpellArming::Free,
            Some(toml_str) => toml::from_str::<SpellArming>(toml_str).map_err(|e| {
                ErrorData::invalid_params(format!("invalid arming TOML: {e}"), None)
            })?,
        };
        self.handle
            .send_command(ControlCommand::TrainSpell {
                name: args.name.clone(),
                arming,
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "spell training started for '{}'",
            args.name
        ))]))
    }
}
