//! Seat-session tools - `list_seat_sessions`, `claim_seat_session`,
//! `release_seat_session`, `add_seat`, `update_seat`, `remove_seat`,
//! `set_seats_visible` (TANGIBLE_OBJECTS_TABLE_TOP_SEAT.md).

use rmcp::ErrorData;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::model::ContentBlock;
use rmcp::tool;
use rmcp::tool_router;
use tangible_objects_runtime::ControlCommand;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::json_pretty;
use crate::tool_args::AddSeatArgs;
use crate::tool_args::ClaimSeatSessionArgs;
use crate::tool_args::ListSeatSessionsArgs;
use crate::tool_args::ReleaseSeatSessionArgs;
use crate::tool_args::RemoveSeatArgs;
use crate::tool_args::SetSeatsVisibleArgs;
use crate::tool_args::UpdateSeatArgs;

/// Seat-session roster and ownership tools.
#[tool_router(router = seats_router, vis = "pub(crate)")]
impl EngineMcp {
    /// Claimed seat sessions with their rosters - `session` filters to
    /// one named session, omitted lists all.
    #[tool(
        description = "List the claimed seat sessions with their rosters; 'session' filters to one named session."
    )]
    async fn list_seat_sessions(
        &self,
        Parameters(args): Parameters<ListSeatSessionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let sessions = self
            .handle
            .seat_sessions()
            .into_iter()
            .filter(|session| match &args.session {
                Some(name) => &session.name == name,
                None => true,
            })
            .collect::<Vec<_>>();
        Ok(CallToolResult::success(vec![ContentBlock::text(
            json_pretty(&sessions)?,
        )]))
    }

    /// Claims or creates a seat session for an owner - the
    /// seat-domain counterpart of `claim_profile`: re-claiming is
    /// idempotent for the same owner, a foreign claim is rejected.
    #[tool(
        description = "Claim a seat session for an owner - created when missing; re-claiming is idempotent for the same owner, rejected for a foreign one; unregister_owner cleanup releases it with all its seats."
    )]
    async fn claim_seat_session(
        &self,
        Parameters(args): Parameters<ClaimSeatSessionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let (command, rx) = ControlCommand::claim_seat_session(args.name, args.owner);
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(Ok(session)) => Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "claimed session '{}' for '{}' ({} seats)",
                session.name,
                session.owner,
                session.seats.len()
            ))])),
            Ok(Err(reason)) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "rejected: {reason}"
            ))])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the claim reply".to_string(),
            )])),
        }
    }

    /// Releases an owner's claim on a seat session - its seats are
    /// removed with it.
    #[tool(
        description = "Release an owner's claim on a seat session - the session's seats are removed with it."
    )]
    async fn release_seat_session(
        &self,
        Parameters(args): Parameters<ReleaseSeatSessionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let (command, rx) = ControlCommand::release_seat_session(args.name, args.owner);
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(Ok(session)) => Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "released session '{}' ({} seats removed)",
                session.name,
                session.seats.len()
            ))])),
            Ok(Err(reason)) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "rejected: {reason}"
            ))])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the release reply".to_string(),
            )])),
        }
    }

    /// Adds a seat to a claimed session - marker-backed (`marker`
    /// set) or virtual (`marker` omitted, `pose` applied).
    #[tool(
        description = "Add a seat to a claimed session - pass 'marker' (pattern ID) for a marker-backed seat whose pose derives from the tracked tangible, omit it for a virtual seat placed via 'pose'."
    )]
    async fn add_seat(
        &self,
        Parameters(args): Parameters<AddSeatArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let (command, rx) =
            ControlCommand::add_seat(args.session, args.owner, args.name, args.marker, args.pose);
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(Ok(seat)) => Ok(CallToolResult::success(vec![ContentBlock::text(
                json_pretty(&seat)?,
            )])),
            Ok(Err(reason)) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "rejected: {reason}"
            ))])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the add-seat reply".to_string(),
            )])),
        }
    }

    /// Mutates a seat addressed by its globally unique ID - omitted
    /// fields keep their value, `clear_marker` turns the seat virtual.
    #[tool(
        description = "Update a seat by its ID - omitted fields keep their value; 'clear_marker: true' removes the marker binding so the seat turns virtual."
    )]
    async fn update_seat(
        &self,
        Parameters(args): Parameters<UpdateSeatArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let marker = if args.clear_marker.unwrap_or(false) {
            Some(None)
        } else {
            args.marker.map(Some)
        };
        let (command, rx) =
            ControlCommand::update_seat(args.seat, args.owner, args.name, marker, args.pose);
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(Ok(seat)) => Ok(CallToolResult::success(vec![ContentBlock::text(
                json_pretty(&seat)?,
            )])),
            Ok(Err(reason)) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "rejected: {reason}"
            ))])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the update-seat reply".to_string(),
            )])),
        }
    }

    /// Removes a seat from its session.
    #[tool(description = "Remove a seat from its session - the owner must own the seat's session.")]
    async fn remove_seat(
        &self,
        Parameters(args): Parameters<RemoveSeatArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let (command, rx) = ControlCommand::remove_seat(args.seat, args.owner);
        self.handle
            .send_command(command)
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        match rx.await {
            Ok(Ok(seat)) => Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "removed seat '{}' (id {})",
                seat.name, seat.id
            ))])),
            Ok(Err(reason)) => Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                "rejected: {reason}"
            ))])),
            Err(_) => Ok(CallToolResult::error(vec![ContentBlock::text(
                "device loop closed the remove-seat reply".to_string(),
            )])),
        }
    }

    /// Shows or hides the seat overlay on the GUI canvas - ignored on
    /// headless daemons.
    #[tool(
        description = "Show or hide the seat overlay on the GUI canvas - ignored on headless daemons."
    )]
    async fn set_seats_visible(
        &self,
        Parameters(args): Parameters<SetSeatsVisibleArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.handle
            .send_command(ControlCommand::SetSeatsVisible(args.visible))
            .await
            .map_err(|_| ErrorData::internal_error("device loop is not running", None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(
            "seat overlay toggled".to_string(),
        )]))
    }
}
