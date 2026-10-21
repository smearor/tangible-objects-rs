//! Recording tools - `recording_status`, `start_recording`,
//! `stop_recording`, `set_record_label`, `recording_artifact`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use std::path::PathBuf;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::json_pretty;
use crate::tool_args::RecordingArtifactArgs;
use crate::tool_args::SetRecordLabelArgs;
use crate::tool_args::StartRecordingArgs;

/// Recording-session tools.
#[tool_router(router = recording_router, vis = "pub(crate)")]
impl EngineMcp {
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

    /// Starts a recording session writing `<base>.jsonl` + `<base>.evtest`.
    #[tool(description = "Start a recording session writing <base>.jsonl and <base>.evtest.")]
    async fn start_recording(
        &self,
        Parameters(args): Parameters<StartRecordingArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::StartRecording {
                base_path: PathBuf::from(args.base_path.clone()),
                label: args.label.clone(),
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
            .send_command(ControlCommand::SetRecordLabel(args.label.clone()))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "record label set to {:?}",
            args.label
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
}
