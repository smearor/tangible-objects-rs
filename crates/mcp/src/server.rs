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

use tangible_objects_model::RecordLabel;
use tangible_objects_model::TangibleId;
use tangible_objects_model::TangibleSetState;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::EngineHandle;

use super::tool_args::GetPoseArgs;
use super::tool_args::RecordingArtifactArgs;
use super::tool_args::SetFilterModeArgs;
use super::tool_args::SetRecordLabelArgs;
use super::tool_args::StartRecordingArgs;
use super::tool_args::TrainProfileArgs;

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
            json_pretty(self.handle.profiles())?,
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

    /// Enables or disables the tangible filter mode.
    #[tool(description = "Enable or disable the tangible filter mode.")]
    async fn set_filter_mode(
        &self,
        Parameters(args): Parameters<SetFilterModeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::SetFilterMode(args.enabled))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "filter mode set to {}",
            args.enabled
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
                URI_PROFILES => json_pretty(handle.profiles())?,
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
