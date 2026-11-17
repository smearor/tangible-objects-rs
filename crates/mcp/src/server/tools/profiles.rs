//! Profile query and ownership tools - `list_profiles`,
//! `find_profile`, `claim_profile`, `release_profile`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_model::Owner;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::json_pretty;
use crate::server::tools::pattern::build_profile;
use crate::tool_args::ClaimProfileArgs;
use crate::tool_args::FindProfileArgs;
use crate::tool_args::ListProfilesArgs;
use crate::tool_args::ReleaseProfileArgs;

/// Loaded-profile query and ownership tools.
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

    /// Compares a candidate profile against the registered patterns -
    /// the `find_profile` check applications run before registering.
    #[tool(
        description = "Check whether a profile with this name/geometry is already registered - returns the colliding pattern's ID, name, owner and whether the fingerprint is identical."
    )]
    async fn find_profile(
        &self,
        Parameters(args): Parameters<FindProfileArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let profile = build_profile(&args.name, &args.points, args.tolerance, None, None, &None)?;
        let (command, rx) = ControlCommand::lookup_profile(profile);
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(lookup) => Ok(CallToolResult::success(vec![ContentBlock::text(
                json_pretty(&lookup)?,
            )])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the profile lookup reply".to_string(),
            )])),
        }
    }

    /// Claims an owner-less profile for an application.
    #[tool(
        description = "Claim an owner-less profile for an owner - the application adopts a statically loaded or released profile; unregister_owner cleanup releases it back to owner-less instead of unregistering it."
    )]
    async fn claim_profile(
        &self,
        Parameters(args): Parameters<ClaimProfileArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let (command, rx) = ControlCommand::claim_profile(args.pattern_id, args.owner, args.enable);
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(Ok(claimed)) => Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "claimed '{}' as pattern {} (enabled: {})",
                claimed.name, claimed.pattern_id, claimed.enabled
            ))])),
            Ok(Err(reason)) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "rejected: {reason}"
            ))])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the claim reply".to_string(),
            )])),
        }
    }

    /// Releases an owner's claim on a profile.
    #[tool(
        description = "Release an owner's claim on a profile - the profile stays registered and keeps matching; it only leaves the owner's cleanup scope."
    )]
    async fn release_profile(
        &self,
        Parameters(args): Parameters<ReleaseProfileArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let (command, rx) =
            ControlCommand::release_profile(args.pattern_id, args.owner, args.disable);
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(Ok(released)) => Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "released '{}' (pattern {}, enabled: {})",
                released.name, released.pattern_id, released.enabled
            ))])),
            Ok(Err(reason)) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "rejected: {reason}"
            ))])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the release reply".to_string(),
            )])),
        }
    }
}
