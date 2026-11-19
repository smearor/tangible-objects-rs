//! Seat-session methods - `seat_sessions`, `claim_seat_session`,
//! `release_seat_session`, `add_seat`, `update_seat`, `remove_seat`
//! (TANGIBLE_OBJECTS_TABLE_TOP_SEAT.md).
//!
//! `SeatPose` has no `zvariant::Type` impl - it travels as the flat
//! `(side, offset, orientation)` triple: `side` indexes
//! `TableSide::ALL` order (0 bottom, 1 left, 2 top, 3 right), a `NaN`
//! `offset` means the edge center (`offset: None`), `orientation` is
//! the free radial angle in radians.

use smearor_commons_geometry::Fraction;
use smearor_commons_geometry::RotationRad;
use smearor_commons_layout::Seat;
use smearor_commons_layout::SeatPose;
use smearor_commons_layout::TableSide;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::SeatId;
use tangible_objects_model::SeatName;
use tangible_objects_model::SeatSessionName;
use tangible_objects_runtime::ControlCommand;
use zbus::fdo;
use zbus::message::Header;

use crate::service::engine_service::EngineService;

/// Marker actions for `update_seat` - the keep/set/clear tri-state
/// `Option<PatternId>` cannot express over D-Bus.
const MARKER_KEEP: u32 = 0;
const MARKER_SET: u32 = 1;
const MARKER_CLEAR: u32 = 2;

/// Rebuilds the wire `(side, offset, orientation)` triple into a
/// [`SeatPose`] - see the module docs for the encoding.
fn build_pose(side: u32, offset: f64, orientation: f64) -> fdo::Result<SeatPose> {
    let side = *TableSide::ALL
        .get(side as usize)
        .ok_or_else(|| fdo::Error::InvalidArgs(format!("side must be 0..=3, got {side}")))?;
    Ok(SeatPose::new(
        Seat {
            side,
            offset: (!offset.is_nan()).then_some(Fraction::new(offset)),
        },
        RotationRad::new(orientation),
    ))
}

/// Claimed seat sessions with their rosters - the JSON-serialized
/// `SeatSession` array (`metrics`/`analyze` JSON-string contract).
pub(crate) async fn seat_sessions(service: &EngineService) -> fdo::Result<String> {
    serde_json::to_string(&service.handle.seat_sessions())
        .map_err(|err| fdo::Error::Failed(format!("failed to serialize seat sessions: {err}")))
}

/// Claims or creates a seat session for `owner` - an empty `owner`
/// defaults to the caller's unique bus name (disconnect cleanup comes
/// with it). Re-claiming is idempotent for the same owner.
pub(crate) async fn claim_seat_session(
    service: &EngineService,
    name: SeatSessionName,
    owner: Owner,
    header: &Header<'_>,
) -> fdo::Result<()> {
    let (command, rx) =
        ControlCommand::claim_seat_session(name, service.session_owner(owner, header));
    service
        .handle
        .send_command(command)
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))?;
    match rx.await {
        Ok(result) => result
            .map(|_| ())
            .map_err(|reason| fdo::Error::Failed(format!("rejected: {reason}"))),
        Err(_) => Err(fdo::Error::Failed(
            "device loop closed the claim reply".to_string(),
        )),
    }
}

/// Releases `owner`'s claim on a seat session - an empty `owner`
/// defaults to the caller's unique bus name. The session's seats are
/// removed with it.
pub(crate) async fn release_seat_session(
    service: &EngineService,
    name: SeatSessionName,
    owner: Owner,
    header: &Header<'_>,
) -> fdo::Result<()> {
    let (command, rx) =
        ControlCommand::release_seat_session(name, service.session_owner(owner, header));
    service
        .handle
        .send_command(command)
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))?;
    match rx.await {
        Ok(result) => result
            .map(|_| ())
            .map_err(|reason| fdo::Error::Failed(format!("rejected: {reason}"))),
        Err(_) => Err(fdo::Error::Failed(
            "device loop closed the release reply".to_string(),
        )),
    }
}

/// Adds a seat to a claimed session and returns its assigned seat ID -
/// marker-backed (`has_marker` with the marker `marker` pattern) or
/// virtual (`has_marker = false`, `has_pose` pose applied). `owner`
/// must own the session; an empty `owner` defaults to the caller's
/// unique bus name.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn add_seat(
    service: &EngineService,
    session: SeatSessionName,
    owner: Owner,
    name: SeatName,
    has_marker: bool,
    marker: PatternId,
    has_pose: bool,
    side: u32,
    offset: f64,
    orientation: f64,
    header: &Header<'_>,
) -> fdo::Result<u32> {
    let pose = has_pose
        .then(|| build_pose(side, offset, orientation))
        .transpose()?;
    let (command, rx) = ControlCommand::add_seat(
        session,
        service.session_owner(owner, header),
        name,
        has_marker.then_some(marker),
        pose,
    );
    service
        .handle
        .send_command(command)
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))?;
    match rx.await {
        Ok(result) => result
            .map(|seat| seat.id.as_u32())
            .map_err(|reason| fdo::Error::Failed(format!("rejected: {reason}"))),
        Err(_) => Err(fdo::Error::Failed(
            "device loop closed the add-seat reply".to_string(),
        )),
    }
}

/// Mutates a seat addressed by its ID and returns the seat ID - an
/// empty `name` keeps the current one, `marker_action` is 0 keep /
/// 1 rebind to `marker` / 2 clear (turns the seat virtual), `has_pose`
/// applies a new pose (honored for virtual seats; a bound marker
/// re-derives it). `owner` must own the seat's session; an empty
/// `owner` defaults to the caller's unique bus name.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn update_seat(
    service: &EngineService,
    seat: SeatId,
    owner: Owner,
    name: SeatName,
    marker_action: u32,
    marker: PatternId,
    has_pose: bool,
    side: u32,
    offset: f64,
    orientation: f64,
    header: &Header<'_>,
) -> fdo::Result<u32> {
    let marker = match marker_action {
        MARKER_KEEP => None,
        MARKER_SET => Some(Some(marker)),
        MARKER_CLEAR => Some(None),
        action => {
            return Err(fdo::Error::InvalidArgs(format!(
                "marker_action must be 0 (keep), 1 (set) or 2 (clear), got {action}"
            )));
        }
    };
    let pose = has_pose
        .then(|| build_pose(side, offset, orientation))
        .transpose()?;
    let name = (!name.as_str().is_empty()).then_some(name);
    let (command, rx) = ControlCommand::update_seat(
        seat,
        service.session_owner(owner, header),
        name,
        marker,
        pose,
    );
    service
        .handle
        .send_command(command)
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))?;
    match rx.await {
        Ok(result) => result
            .map(|seat| seat.id.as_u32())
            .map_err(|reason| fdo::Error::Failed(format!("rejected: {reason}"))),
        Err(_) => Err(fdo::Error::Failed(
            "device loop closed the update-seat reply".to_string(),
        )),
    }
}

/// Removes a seat from its session - `owner` must own the seat's
/// session; an empty `owner` defaults to the caller's unique bus name.
pub(crate) async fn remove_seat(
    service: &EngineService,
    seat: SeatId,
    owner: Owner,
    header: &Header<'_>,
) -> fdo::Result<u32> {
    let (command, rx) = ControlCommand::remove_seat(seat, service.session_owner(owner, header));
    service
        .handle
        .send_command(command)
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))?;
    match rx.await {
        Ok(result) => result
            .map(|seat| seat.id.as_u32())
            .map_err(|reason| fdo::Error::Failed(format!("rejected: {reason}"))),
        Err(_) => Err(fdo::Error::Failed(
            "device loop closed the remove-seat reply".to_string(),
        )),
    }
}

/// Shows or hides the seat overlay on the GUI canvas - ignored on
/// headless daemons.
pub(crate) async fn set_seats_visible(service: &EngineService, visible: bool) -> fdo::Result<()> {
    service
        .handle
        .send_command(ControlCommand::SetSeatsVisible(visible))
        .await
        .map_err(|_| fdo::Error::Failed("device loop is not running".to_string()))
}
