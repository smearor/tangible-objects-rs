//! Completion tools - `get_completion`, `set_completion`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_model::CompletionMode;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::json_pretty;
use crate::tool_args::SetCompletionArgs;

/// Missing-pin completion tools.
#[tool_router(router = completion_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Active missing-pin completion configuration.
    #[tool(
        description = "Get the missing-pin completion config: claim radius factor, max missing pins, persist flag and mode."
    )]
    async fn get_completion(&self) -> Result<CallToolResult, ErrorData> {
        let snapshot = self.handle.snapshot();
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&snapshot.completion)?,
        )]))
    }

    /// Replaces the missing-pin completion configuration.
    #[tool(
        description = "Set the missing-pin completion config: mode 'match' or 'exact', claim radius factor, max missing pins (1|2), persist flag."
    )]
    async fn set_completion(
        &self,
        Parameters(args): Parameters<SetCompletionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut config = self.handle.snapshot().completion.unwrap_or_default();
        if let Some(mode) = &args.mode {
            config.mode = mode.parse::<CompletionMode>().map_err(|_| {
                ErrorData::invalid_params("mode must be one of: match, exact", None)
            })?;
        }
        if let Some(radius) = args.radius_factor {
            config.radius_factor = radius;
        }
        if let Some(missing) = args.max_missing {
            if !(1..=2).contains(&missing) {
                return Err(ErrorData::invalid_params(
                    "max_missing must be 1 or 2",
                    None,
                ));
            }
            config.max_missing = missing;
        }
        if let Some(persist) = args.persist {
            config.persist = persist;
        }
        if let Some(factor) = args.affinity_factor {
            config.affinity_factor = factor.into();
        }
        if let Some(alpha) = args.smoothing_alpha {
            config.smoothing_alpha = alpha.into();
        }
        self.handle
            .send_command(ControlCommand::SetCompletion(config))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "completion set: mode={} radius_factor={} max_missing={} persist={} affinity={} smoothing={}",
            config.mode,
            config.radius_factor,
            config.max_missing,
            config.persist,
            config.affinity_factor,
            config.smoothing_alpha
        ))]))
    }
}
