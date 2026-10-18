use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::model::ListResourcesResult;
use rmcp::model::PaginatedRequestParams;
use rmcp::model::ReadResourceRequestParams;
use rmcp::model::ReadResourceResponse;
use rmcp::model::ReadResourceResult;
use rmcp::model::Resource;
use rmcp::model::ResourceContents;
use rmcp::model::ServerCapabilities;
use rmcp::model::ServerConfig;
use rmcp::service::RequestContext;
use rmcp::service::RoleServer;
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use std::path::PathBuf;

use tangible_objects_model::CompletionMode;
use tangible_objects_model::FilterMode;
use tangible_objects_model::Owner;
use tangible_objects_model::RecordLabel;
use tangible_objects_model::Registration;
use tangible_objects_model::TangibleId;
use tangible_objects_model::TangibleSetState;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::EngineHandle;

use super::tool_args::AnalyzeArgs;
use super::tool_args::GetPoseArgs;
use super::tool_args::RecordingArtifactArgs;
use super::tool_args::RegisterFieldSetArgs;
use super::tool_args::RegisterPatternArgs;
use super::tool_args::SetCompletionArgs;
use super::tool_args::SetFilterModeArgs;
use super::tool_args::SetRecordLabelArgs;
use super::tool_args::ShowMessageArgs;
use super::tool_args::StartRecordingArgs;
use super::tool_args::TrainProfileArgs;
use super::tool_args::UnregisterFieldSetArgs;
use super::tool_args::UnregisterOwnerArgs;
use super::tool_args::UnregisterPatternArgs;
use super::tool_args::UpdateProfileArgs;

/// Resource URI listing the currently tracked tangibles.
pub const URI_TANGIBLES: &str = "tangible://tangibles";
/// Resource URI listing the active touch points.
pub const URI_TOUCH_POINTS: &str = "tangible://touch-points";
/// Resource URI listing the configured sets with activation state.
pub const URI_SETS: &str = "tangible://sets";
/// Resource URI listing the loaded figure profiles.
pub const URI_PROFILES: &str = "tangible://profiles";

/// MCP server bridging the [`EngineHandle`] to tools and resources.
#[derive(Clone)]
pub struct EngineMcp {
    handle: EngineHandle,
}

#[tool_router]
impl EngineMcp {
    /// Creates the MCP server for the given runtime handle.
    pub fn new(handle: EngineHandle) -> Self {
        Self { handle }
    }

    /// All currently tracked tangibles with smoothed pose.
    #[tool(description = "List all tangibles currently tracked on the multitouch surface.")]
    async fn list_tangibles(&self) -> Result<CallToolResult, ErrorData> {
        let snapshot = self.handle.snapshot();
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&snapshot.tracked)?,
        )]))
    }

    /// Center and rotation of one tangible by tracking ID.
    #[tool(description = "Get center and rotation of a tangible by tracking ID.")]
    async fn get_pose(
        &self,
        Parameters(args): Parameters<GetPoseArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let snapshot = self.handle.snapshot();
        let tangible_id = TangibleId::from(args.tangible_id);
        let Some(tangible) = snapshot.tracked.iter().find(|t| t.id == tangible_id) else {
            return Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "no tangible with id {}",
                args.tangible_id
            ))]));
        };
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(tangible)?,
        )]))
    }

    /// Loaded figure profiles.
    #[tool(description = "List the loaded figure profiles.")]
    async fn list_profiles(&self) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&self.handle.profiles())?,
        )]))
    }

    /// Attached input device info (name + axis calibration).
    #[tool(
        description = "Get the attached touchscreen's name and axis calibration - needed to convert this host's raw recordings to another device."
    )]
    async fn device_info(&self) -> Result<CallToolResult, ErrorData> {
        let snapshot = self.handle.snapshot();
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&snapshot.device)?,
        )]))
    }

    /// Live recording-session status.
    #[tool(
        description = "Get the recording status: active flag, label, recorded/dropped frame counters."
    )]
    async fn recording_status(&self) -> Result<CallToolResult, ErrorData> {
        let snapshot = self.handle.snapshot();
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&snapshot.recording)?,
        )]))
    }

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

    /// Returns the accumulated session metrics.
    #[tool(
        description = "Get session recognition metrics: frame uptime, tangible re-ID churn, exact/partial match frames, claimed pins, per-slot tracking-ID churn."
    )]
    async fn get_metrics(&self) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&self.handle.snapshot().metrics)?,
        )]))
    }

    /// Resets the session metric counters.
    #[tool(
        description = "Reset the accumulated session metrics - call between automated test runs to isolate each series."
    )]
    async fn reset_metrics(&self) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::ResetMetrics)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(
            "metrics reset".to_string(),
        )]))
    }

    /// Pushes a user-facing message into the GUI overlay.
    #[tool(
        description = "Show a user-facing message in the GUI overlay (e.g. 'Legen Sie Figur 10 auf'); an empty text clears it."
    )]
    async fn show_message(
        &self,
        Parameters(args): Parameters<ShowMessageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::ShowMessage(args.text))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(
            "message shown".to_string(),
        )]))
    }

    /// Runs the offline deviation analysis over a recording artifact.
    #[tool(
        description = "Analyze a recording (.jsonl): per-pattern per-distance deviation statistics - mean/p99/max deviation, over-tolerance counts and which pin pair most often blocked the match."
    )]
    async fn analyze_recording(
        &self,
        Parameters(args): Parameters<AnalyzeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let recording = tangible_objects_analysis::Recording::load(std::path::Path::new(
            &args.path,
        ))
        .map_err(|e| ErrorData::invalid_params(format!("recording unreadable: {e}"), None))?;
        let report = if args.mode == Some(tangible_objects_model::AnalysisMode::Contacts) {
            let dir = args.profiles_dir.as_deref().unwrap_or("profiles");
            let loaded = tangible_objects_model::ProfileFile::load_dir(std::path::Path::new(dir));
            let patterns = loaded.into_patterns();
            if patterns.is_empty() {
                return Err(ErrorData::invalid_params(
                    format!("no profiles loaded from {dir}"),
                    None,
                ));
            }
            tangible_objects_analysis::DeviationReport::analyze_contacts(&recording, &patterns)
        } else {
            tangible_objects_analysis::DeviationReport::analyze(&recording)
        };
        Ok(CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&report)
                .map_err(|e| ErrorData::internal_error(format!("serialize: {e}"), None))?,
        )]))
    }

    /// Registers a figure profile at runtime.
    #[tool(
        description = "Register a figure profile by name, flat pin coordinates [x0,y0,...] mm and tolerance mm. Fingerprint collisions with loaded profiles are rejected by the daemon."
    )]
    async fn register_pattern(
        &self,
        Parameters(args): Parameters<RegisterPatternArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        use tangible_objects_model::Millimeters;
        use tangible_objects_model::PatternTolerance;
        use tangible_objects_model::ProfileFile;
        use tangible_objects_model::Vec2;

        if !args.points.len().is_multiple_of(2) || args.points.is_empty() {
            return Err(ErrorData::invalid_params(
                "points must be a non-empty even-length flat list",
                None,
            ));
        }
        let points: Vec<Vec2<Millimeters>> = args
            .points
            .chunks_exact(2)
            .map(|pair| Vec2::new(pair[0], pair[1]))
            .collect();
        let mut profile =
            ProfileFile::from_points(&args.name, &points, PatternTolerance::new(args.tolerance));
        profile.spawn_on_partial = args.spawn_on_partial.unwrap_or(false);
        if let Some(stability) = args.spawn_stability {
            profile.spawn_stability = tangible_objects_model::FrameCount::new(stability);
        }
        self.handle
            .send_command(ControlCommand::Register {
                registration: Box::new(Registration::Pattern(profile)),
                owner: args.owner.map(Owner::new),
                lifetime: parse_lifetime(&args.lifetime)?,
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "registration queued: {}",
            args.name
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
                id: tangible_objects_model::PatternId::new(args.pattern_id),
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "unregistered pattern {}",
            args.pattern_id
        ))]))
    }

    /// Registers an occupancy field set from TOML.
    #[tool(
        description = "Register an occupancy field set from TOML (name, optional stability/[window], [[fields]] cells)."
    )]
    async fn register_field_set(
        &self,
        Parameters(args): Parameters<RegisterFieldSetArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let set = toml::from_str::<tangible_objects_model::FieldSet>(&args.toml)
            .map_err(|e| ErrorData::invalid_params(format!("invalid field set TOML: {e}"), None))?;
        self.handle
            .send_command(ControlCommand::Register {
                registration: Box::new(Registration::FieldSet(set)),
                owner: args.owner.map(Owner::new),
                lifetime: parse_lifetime(&args.lifetime)?,
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(
            "field set registration queued",
        )]))
    }

    /// Replaces a registered pattern's geometry while keeping its ID.
    #[tool(
        description = "Replace a registered pattern's geometry (name, points, tolerance) while keeping its pattern ID. Applies to new matches."
    )]
    async fn update_profile(
        &self,
        Parameters(args): Parameters<UpdateProfileArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        use tangible_objects_model::Millimeters;
        use tangible_objects_model::PatternId;
        use tangible_objects_model::PatternTolerance;
        use tangible_objects_model::ProfileFile;
        use tangible_objects_model::Vec2;

        if !args.points.len().is_multiple_of(2) || args.points.is_empty() {
            return Err(ErrorData::invalid_params(
                "points must be a non-empty even-length flat list",
                None,
            ));
        }
        let points: Vec<Vec2<Millimeters>> = args
            .points
            .chunks_exact(2)
            .map(|pair| Vec2::new(pair[0], pair[1]))
            .collect();
        let mut profile =
            ProfileFile::from_points(&args.name, &points, PatternTolerance::new(args.tolerance));
        profile.spawn_on_partial = args.spawn_on_partial.unwrap_or(false);
        if let Some(stability) = args.spawn_stability {
            profile.spawn_stability = tangible_objects_model::FrameCount::new(stability);
        }
        self.handle
            .send_command(ControlCommand::UpdateProfile {
                id: PatternId::new(args.pattern_id),
                profile: Box::new(profile),
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "updated pattern {}",
            args.pattern_id
        ))]))
    }

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

    /// Reads a recording artifact file from the daemon host.
    #[tool(
        description = "Download a recording artifact (*.jsonl, *.evtest, *.dataset.jsonl) produced by this daemon - remote recording pull."
    )]
    async fn recording_artifact(
        &self,
        Parameters(args): Parameters<RecordingArtifactArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = PathBuf::from(&args.path);
        let valid_ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "jsonl" || e == "evtest")
            .unwrap_or(false);
        let valid_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with(".dataset.jsonl"))
            .unwrap_or(false);
        if path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
            || !(valid_ext || valid_name)
        {
            return Ok(CallToolResult::error(vec![ContentBlock::text(
                "only recording artifacts (*.jsonl, *.evtest, *.dataset.jsonl) without '..' are readable",
            )]));
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => Ok(CallToolResult::success(vec![ContentBlock::text(content)])),
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "cannot read {}: {e}",
                path.display()
            ))])),
        }
    }

    /// Switches the filter proxy's suppression policy.
    #[tool(
        description = "Set the filter proxy mode: off, immediate, gated, guarded or tangibles-only."
    )]
    async fn set_filter_mode(
        &self,
        Parameters(args): Parameters<SetFilterModeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let mode = args.mode.parse::<FilterMode>().map_err(|_| {
            ErrorData::invalid_params(
                "mode must be one of: off, immediate, gated, guarded, tangibles-only",
                None,
            )
        })?;
        let stability = args.stability.map(tangible_objects_model::FrameCount::new);
        self.handle
            .send_command(ControlCommand::SetFilterMode { mode, stability })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "filter mode set to {mode}"
        ))]))
    }

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

    /// Starts a recording session writing `<base>.jsonl` + `<base>.evtest`.
    #[tool(description = "Start a recording session writing <base>.jsonl and <base>.evtest.")]
    async fn start_recording(
        &self,
        Parameters(args): Parameters<StartRecordingArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::StartRecording {
                base_path: PathBuf::from(args.base_path.clone()),
                label: args.label.clone().map(RecordLabel::new),
            })
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "recording started at '{}'",
            args.base_path
        ))]))
    }

    /// Stops the active recording session.
    #[tool(description = "Stop the active recording session.")]
    async fn stop_recording(&self) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::StopRecording)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(
            "recording stopped",
        )]))
    }

    /// Changes the ground-truth label for subsequent recorded frames.
    #[tool(
        description = "Set the ground-truth label for subsequent recorded frames; null clears it."
    )]
    async fn set_record_label(
        &self,
        Parameters(args): Parameters<SetRecordLabelArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::SetRecordLabel(
                args.label.clone().map(RecordLabel::new),
            ))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "record label set to {:?}",
            args.label
        ))]))
    }
}

#[tool_handler]
impl ServerHandler for EngineMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_instructions(
            "Tangible object detection engine on a multitouch surface. \
             Tools expose the live detection state and control commands; \
             resources under tangible:// provide read-only snapshots.",
        )
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, ErrorData>> + Send + '_ {
        std::future::ready(Ok(ListResourcesResult::with_all_items(vec![
            resource(URI_TANGIBLES, "tangibles", "Currently tracked tangibles"),
            resource(URI_TOUCH_POINTS, "touch-points", "Active touch points"),
            resource(
                URI_SETS,
                "sets",
                "Tangible Object Sets with activation state",
            ),
            resource(URI_PROFILES, "profiles", "Loaded figure profiles"),
        ])))
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResponse, ErrorData>> + Send + '_ {
        let handle = self.handle.clone();
        async move {
            let body = match request.uri.as_str() {
                URI_TANGIBLES => json_pretty(&handle.snapshot().tracked)?,
                URI_TOUCH_POINTS => json_pretty(&handle.snapshot().touches)?,
                URI_SETS => {
                    let active = handle.snapshot().active_sets;
                    json_pretty(
                        &handle
                            .sets()
                            .iter()
                            .map(|set| TangibleSetState::from((set, active.contains(&set.id))))
                            .collect::<Vec<_>>(),
                    )?
                }
                URI_PROFILES => json_pretty(&handle.profiles())?,
                uri => {
                    return Err(ErrorData::resource_not_found(
                        format!("unknown resource: {uri}"),
                        None,
                    ));
                }
            };
            Ok(ReadResourceResult::new(vec![
                ResourceContents::text(body, request.uri).with_mime_type("application/json"),
            ])
            .into())
        }
    }
}

/// Serializes a value to pretty JSON for tool/resource output.
/// Parses the optional `lifetime` tool argument (`session`/`persistent`).
fn parse_lifetime(
    lifetime: &Option<String>,
) -> Result<tangible_objects_model::Lifetime, ErrorData> {
    lifetime
        .as_deref()
        .map(str::parse)
        .transpose()
        .map(|parsed| parsed.unwrap_or_default())
        .map_err(|_| ErrorData::invalid_params("lifetime must be 'session' or 'persistent'", None))
}

fn json_pretty<T: serde::Serialize + ?Sized>(value: &T) -> Result<String, ErrorData> {
    serde_json::to_string_pretty(value)
        .map_err(|err| ErrorData::internal_error(format!("serialization failed: {err}"), None))
}

/// Describes one `tangible://` resource in `resources/list`.
fn resource(uri: &str, name: &str, description: &str) -> Resource {
    Resource::new(uri, name)
        .with_description(description)
        .with_mime_type("application/json")
}
