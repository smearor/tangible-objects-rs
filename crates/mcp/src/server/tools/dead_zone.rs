//! Dead-zone tools - `register_dead_zone`, `unregister_dead_zone`,
//! `dead_zones`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use smearor_commons_geometry::Vec2;
use tangible_objects_model::DEFAULT_DEAD_ZONE_RADIUS;
use tangible_objects_model::DeadZone;
use tangible_objects_model::Registration;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::json_pretty;
use crate::server::helpers::parse_lifetime;
use crate::tool_args::RegisterDeadZoneArgs;
use crate::tool_args::UnregisterDeadZoneArgs;

/// Dead-zone registration and query tools.
#[tool_router(router = dead_zone_router, vis = "pub(crate)")]
impl EngineMcp {
    /// All active dead zones - blacklisted surface positions whose
    /// contacts are dropped at ingest.
    #[tool(
        description = "List the active dead zones: blacklisted surface spots (name, center mm, radius mm) whose contacts are suppressed."
    )]
    async fn dead_zones(&self) -> Result<CallToolResult, ErrorData> {
        let snapshot = self.handle.snapshot();
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&snapshot.dead_zones)?,
        )]))
    }

    /// Registers a dead zone: a circular blacklist spot (name, x/y mm,
    /// radius mm - default 10) whose contacts are dropped at ingest.
    /// Ghost contacts never reach tracking, matching or fields.
    /// `lifetime: "persistent"` writes the zone to the runtime store
    /// so it reloads on daemon start.
    #[tool(
        description = "Blacklist a circular surface spot (name, x/y mm, radius mm - default 10): contacts inside are dropped at ingest. lifetime `persistent` reloads it on daemon start."
    )]
    async fn register_dead_zone(
        &self,
        Parameters(args): Parameters<RegisterDeadZoneArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let zone = DeadZone::new(
            args.name,
            Vec2::from_mm(args.x, args.y),
            args.radius.unwrap_or(DEFAULT_DEAD_ZONE_RADIUS),
        );
        let (command, rx) = ControlCommand::register(
            Registration::DeadZone(zone),
            args.owner,
            parse_lifetime(&args.lifetime)?,
        );
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(Ok(_)) => Ok(CallToolResult::success(vec![ContentBlock::text(
                "dead zone registered".to_string(),
            )])),
            Ok(Err(reason)) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "rejected: {reason}"
            ))])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the registration reply".to_string(),
            )])),
        }
    }

    /// Removes a dead zone by name.
    #[tool(description = "Unregister a dead zone by name.")]
    async fn unregister_dead_zone(
        &self,
        Parameters(args): Parameters<UnregisterDeadZoneArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::UnregisterDeadZone {
                name: args.name.clone(),
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "unregistered dead zone {}",
            args.name
        ))]))
    }
}
