//! GUI tools - `set_gui_visible`.

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_model::DisplayRotation;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::tool_args::SetBackgroundAlphaArgs;
use crate::tool_args::SetDeadZonesVisibleArgs;
use crate::tool_args::SetDisplayRotationArgs;
use crate::tool_args::SetEdgeMarginVisibleArgs;
use crate::tool_args::SetFieldsVisibleArgs;
use crate::tool_args::SetGuiVisibleArgs;
use crate::tool_args::SetPositionsVisibleArgs;
use crate::tool_args::SetRulerVisibleArgs;
use crate::tool_args::SetTouchPassthroughArgs;

/// Daemon-GUI visibility tools.
#[tool_router(router = gui_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Shows or hides the daemon-embedded GUI window (`--gui` mode).
    #[tool(
        description = "Show or hide the daemon-embedded GUI window (--gui mode); no-op for headless daemons."
    )]
    async fn set_gui_visible(
        &self,
        Parameters(args): Parameters<SetGuiVisibleArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::SetGuiVisible(args.visible))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "gui visible: {}",
            args.visible
        ))]))
    }

    /// Quarter-turn the GUI canvas renders (0/90/180/270 clockwise) -
    /// needed when the desktop shows the touchscreen rotated.
    #[tool(
        description = "Set the daemon-embedded GUI canvas rotation (--gui mode): '0', '90', '180' or '270' degrees clockwise from the viewer's perspective; no-op for headless daemons."
    )]
    async fn set_display_rotation(
        &self,
        Parameters(args): Parameters<SetDisplayRotationArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let rotation: DisplayRotation = args
            .rotation
            .parse()
            .map_err(|message: String| ErrorData::invalid_params(message, None))?;
        self.handle
            .send_command(ControlCommand::SetDisplayRotation(rotation))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "display rotation: {rotation}"
        ))]))
    }

    /// Shows or hides the dead-zone overlay on the daemon-embedded GUI
    /// canvas (`--gui` mode); no-op for headless daemons.
    #[tool(
        description = "Show or hide the dead-zone overlay on the daemon-embedded GUI canvas (--gui mode); no-op for headless daemons."
    )]
    async fn set_dead_zones_visible(
        &self,
        Parameters(args): Parameters<SetDeadZonesVisibleArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::SetDeadZonesVisible(args.visible))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "dead-zone overlay visible: {}",
            args.visible
        ))]))
    }

    /// Sets the GUI canvas background opacity in percent, 0-100
    /// (`--gui` mode); no-op for headless daemons.
    #[tool(
        description = "Set the daemon-embedded GUI canvas background opacity in percent, 0-100 (--gui mode); no-op for headless daemons."
    )]
    async fn set_background_alpha(
        &self,
        Parameters(args): Parameters<SetBackgroundAlphaArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::SetBackgroundAlpha(args.percent.into()))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "background alpha: {} %",
            args.percent
        ))]))
    }

    /// Shows or hides the edge-margin overlay on the daemon-embedded
    /// GUI canvas (`--gui` mode); no-op for headless daemons.
    #[tool(
        description = "Show or hide the edge-margin overlay on the daemon-embedded GUI canvas (--gui mode); no-op for headless daemons."
    )]
    async fn set_edge_margin_visible(
        &self,
        Parameters(args): Parameters<SetEdgeMarginVisibleArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::SetEdgeMarginVisible(args.visible))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "edge-margin overlay visible: {}",
            args.visible
        ))]))
    }

    /// Shows or hides the field-set overlay on the daemon-embedded
    /// GUI canvas (`--gui` mode); no-op for headless daemons.
    #[tool(
        description = "Show or hide the field-set overlay on the daemon-embedded GUI canvas (--gui mode); no-op for headless daemons."
    )]
    async fn set_fields_visible(
        &self,
        Parameters(args): Parameters<SetFieldsVisibleArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::SetFieldsVisible(args.visible))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "field-set overlay visible: {}",
            args.visible
        ))]))
    }

    /// Shows or hides the ruler overlay on the daemon-embedded GUI
    /// canvas (`--gui` mode); no-op for headless daemons.
    #[tool(
        description = "Show or hide the ruler overlay on the daemon-embedded GUI canvas (--gui mode); no-op for headless daemons."
    )]
    async fn set_ruler_visible(
        &self,
        Parameters(args): Parameters<SetRulerVisibleArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::SetRulerVisible(args.visible))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "ruler overlay visible: {}",
            args.visible
        ))]))
    }

    /// Shows or hides the positions overlay on the daemon-embedded
    /// GUI canvas (`--gui` mode); no-op for headless daemons.
    #[tool(
        description = "Show or hide the positions overlay on the daemon-embedded GUI canvas (--gui mode); no-op for headless daemons."
    )]
    async fn set_positions_visible(
        &self,
        Parameters(args): Parameters<SetPositionsVisibleArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::SetPositionsVisible(args.visible))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "positions overlay visible: {}",
            args.visible
        ))]))
    }

    /// Lets touches pass through the daemon-embedded GTK window to
    /// the desktop beneath, except where GUI elements are hit
    /// (`--gui` mode, Wayland only); no-op for headless daemons.
    #[tool(
        description = "Let touches pass through the daemon-embedded GTK window to the desktop beneath, except where GUI elements are hit (--gui mode, Wayland only); no-op for headless daemons."
    )]
    async fn set_touch_passthrough(
        &self,
        Parameters(args): Parameters<SetTouchPassthroughArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::SetTouchPassthrough(args.enabled))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(format!(
            "touch passthrough enabled: {}",
            args.enabled
        ))]))
    }
}
