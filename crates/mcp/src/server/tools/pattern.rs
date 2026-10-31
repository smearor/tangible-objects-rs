//! Pattern-registration tools - `register_pattern`,
//! `update_profile`, `unregister_pattern`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::Registration;
use tangible_objects_model::StageName;
use tangible_objects_model::Vec2;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::parse_lifetime;
use crate::tool_args::RegisterPatternArgs;
use crate::tool_args::UnregisterPatternArgs;
use crate::tool_args::UpdateProfileArgs;

/// Profile building shared by `register_pattern`/`update_profile` -
/// validates the flat pin list into `Vec2<Millimeters>` pairs and
/// fills the spawn/stage options.
fn build_profile(
    name: &str,
    points: &[f64],
    tolerance: PatternTolerance,
    spawn_on_partial: Option<bool>,
    spawn_stability: Option<u64>,
    stage: &Option<StageName>,
) -> Result<ProfileFile, ErrorData> {
    if !points.len().is_multiple_of(2) || points.is_empty() {
        return Err(ErrorData::invalid_params(
            "points must be a non-empty even-length flat list",
            None,
        ));
    }
    let points: Vec<Vec2<Millimeters>> = points
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| Vec2::new(pair[0], pair[1]))
        .collect();
    let mut profile = ProfileFile::from_points(name, &points, tolerance);
    profile.spawn_on_partial = spawn_on_partial.unwrap_or(false);
    profile.stage = stage.clone();
    if let Some(stability) = spawn_stability {
        profile.spawn_stability = tangible_objects_model::FrameCount::new(stability);
    }
    Ok(profile)
}

/// Pattern-registration tools.
#[tool_router(router = pattern_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Registers a figure profile at runtime.
    #[tool(
        description = "Register a figure profile by name, flat pin coordinates [x0,y0,...] mm and tolerance mm. Fingerprint collisions with loaded profiles are rejected by the daemon."
    )]
    async fn register_pattern(
        &self,
        Parameters(args): Parameters<RegisterPatternArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let profile = build_profile(
            &args.name,
            &args.points,
            args.tolerance,
            args.spawn_on_partial,
            args.spawn_stability,
            &args.stage,
        )?;
        let (command, rx) = ControlCommand::register(
            Registration::Pattern(profile),
            args.owner,
            parse_lifetime(&args.lifetime)?,
        );
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(Ok(())) => Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "registered: {}",
                args.name
            ))])),
            Ok(Err(reason)) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "rejected: {reason}"
            ))])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the registration reply".to_string(),
            )])),
        }
    }

    /// Replaces a registered pattern's geometry while keeping its ID.
    #[tool(
        description = "Replace a registered pattern's geometry (name, points, tolerance) while keeping its pattern ID. Applies to new matches."
    )]
    async fn update_profile(
        &self,
        Parameters(args): Parameters<UpdateProfileArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let profile = build_profile(
            &args.name,
            &args.points,
            args.tolerance,
            args.spawn_on_partial,
            args.spawn_stability,
            &args.stage,
        )?;
        self.handle
            .send_command(ControlCommand::UpdateProfile {
                id: args.pattern_id,
                profile: Box::new(profile),
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "updated pattern {}",
            args.pattern_id
        ))]))
    }

    /// Removes a dynamically registered pattern.
    #[tool(description = "Unregister a dynamically registered pattern by pattern ID.")]
    async fn unregister_pattern(
        &self,
        Parameters(args): Parameters<UnregisterPatternArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::UnregisterPattern {
                id: args.pattern_id,
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "unregistered pattern {}",
            args.pattern_id
        ))]))
    }
}
