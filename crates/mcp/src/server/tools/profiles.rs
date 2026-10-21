//! Profile query tools - `list_profiles`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_model::Owner;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::json_pretty;
use crate::tool_args::ListProfilesArgs;

/// Loaded-profile query tools.
#[tool_router(router = profiles_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Loaded figure profiles - same `owner`/`static_only` filters as
    /// the GraphQL `profiles` query.
    #[tool(
        description = "List the loaded figure profiles; 'owner' filters to one application, 'static_only' keeps only owner-less static profiles."
    )]
    async fn list_profiles(
        &self,
        Parameters(args): Parameters<ListProfilesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let profiles = self
            .handle
            .profiles()
            .into_iter()
            .filter(|info| match &args.owner {
                Some(owner) => info.owner.as_ref().map(Owner::as_str) == Some(owner.as_str()),
                None => true,
            })
            .filter(|info| !(args.static_only.unwrap_or(false) && info.owner.is_some()))
            .collect::<Vec<_>>();
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&profiles)?,
        )]))
    }
}
