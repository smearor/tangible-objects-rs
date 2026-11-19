//! `seats` example - TANGIBLE_OBJECTS_TABLE_TOP_SEAT.md end-to-end: a
//! floating-board card passed between tabletop seats the daemon tracks.
//!
//! - Claims the `seats-demo` session and fills it with four virtual
//!   seats, one centered on every table side.
//! - A timer (or the Pass button) hands the card to the next occupied
//!   seat via `rotate_to_next_occupied_seat`; the camera follows with
//!   the Prezi-style `Transit` flight (`goto_card`).
//! - A daemon `TANGIBLE_ADDED` event claims the marker profile and
//!   rebinds the first virtual seat to it (`updateSeat`) - the virtual
//!   seat becomes marker-backed live, and the pass roster follows the
//!   marker's edge position.
//!
//! Start a daemon first (`--graphql-listen 127.0.0.1:8080`); the
//! endpoint resolves like the CLI (`TANGIBLE_ENDPOINT`/`client.toml`).

use floating_board::CardConfig;
use floating_board::FloatingBoard;
use floating_board::GotoTransition;
use floating_board::TurnDirection;
use futures_lite::StreamExt;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::glib;
use gtk4::prelude::*;
use smearor_commons_geometry::Fraction;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Rect;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::Vec2;
use smearor_commons_layout::Seat;
use smearor_commons_layout::SeatPose;
use smearor_commons_layout::TableSide;
use smearor_commons_layout::View;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tangible_objects_client::Endpoint;
use tangible_objects_client::EventStreamQuery;
use tangible_objects_client::GraphQLClient;
use tangible_objects_client::ProfileQuery;
use tangible_objects_client::SeatUpdate;
use tangible_objects_client::model::AddSeatResult;
use tangible_objects_client::model::ClaimProfileResult;
use tangible_objects_client::model::ClaimSeatSessionResult;
use tangible_objects_client::model::EventKind;
use tangible_objects_client::model::Owner;
use tangible_objects_client::model::SeatId;
use tangible_objects_client::model::SeatMarkerState;
use tangible_objects_client::model::SeatName;
use tangible_objects_client::model::SeatSessionName;
use tangible_objects_client::model::StreamEvent;
use tangible_objects_client::model::TableTopSeat;
use tangible_objects_client::model::UpdateSeatResult;
use tokio::runtime::Runtime;
use tracing::info;
use tracing::warn;

/// The example's registration owner - every claim is cleaned up by
/// `unregister_owner` on quit.
const OWNER: &str = "seats-example";
/// Session name - shows up in the daemon GUI's Seats card.
const SESSION: &str = "seats-demo";

/// The four initial virtual seats: centered on every side, gazing
/// inward (surface-mm convention: `Bottom` faces `-y`, `Left` `+x`,
/// `Top` `+y`, `Right` `-x`).
const VIRTUAL_SEATS: [(TableSide, f64); 4] = [
    (TableSide::Bottom, -std::f64::consts::FRAC_PI_2),
    (TableSide::Left, 0.0),
    (TableSide::Top, std::f64::consts::FRAC_PI_2),
    (TableSide::Right, std::f64::consts::PI),
];

/// A synthesized virtual seat for the no-daemon fallback roster and
/// the initial drawing - the daemon's seats replace it on the first
/// roster snapshot.
fn virtual_seat(id: u32, side: TableSide, gaze: f64) -> TableTopSeat {
    TableTopSeat {
        id: SeatId(id),
        name: SeatName::new(format!("seat-{side}")),
        session: SeatSessionName::new(SESSION),
        owner: Owner::new(OWNER),
        marker: None,
        tangible: None,
        pose: SeatPose::new(Seat::at(side, Fraction::HALF), RotationRad::new(gaze)),
        state: SeatMarkerState::Virtual,
    }
}

/// The camera flight's duration - matches the card's default
/// `animation_speed_ms`, so glide and Transit land together.
const GOTO_FLIGHT_MS: u64 = 500;
/// Marker radius in pixels.
const MARKER_RADIUS: f64 = 14.0;
/// Distance of the marker center from the window edge, in pixels.
const MARKER_INSET: f64 = 28.0;
/// Gaze indicator length in pixels.
const GAZE_LENGTH: f64 = 22.0;
/// Marker colors cycled over anchored seats (virtual seats draw gray).
const SEAT_COLORS: [(f64, f64, f64); 4] = [
    (0.21, 0.52, 0.89),
    (0.20, 0.82, 0.35),
    (0.96, 0.83, 0.18),
    (0.57, 0.25, 0.67),
];

/// Canvas position of a seat's marker: `offset` along the edge
/// (0 = left/top end), `MARKER_INSET` inside the window - the same
/// edge-point rule the daemon GUI's seat overlay uses.
fn marker_position(seat: &Seat, width: f64, height: f64) -> (f64, f64) {
    let offset = seat.offset.unwrap_or(Fraction::HALF).as_f64();
    match seat.side {
        TableSide::Bottom => (width * offset, height - MARKER_INSET),
        TableSide::Left => (MARKER_INSET, height * offset),
        TableSide::Top => (width * offset, MARKER_INSET),
        TableSide::Right => (width - MARKER_INSET, height * offset),
    }
}

/// The camera target for a passed card: a region centered partway
/// from the seat's edge marker toward the window center, so the
/// marker ring, its name and the incoming card share the frame -
/// the card rests at the seat edge, not under the crosshair.
///
/// The marker point and the region's size are both derived from the
/// fixed workspace extent, never from the current camera: mapping a
/// widget point through `View::to_workspace` answers "which workspace
/// spot sits under that screen pixel *right now*" - once the view is
/// zoomed in, every next target lands inside the already-shown region
/// and the framing degenerates to the middle of the table.
fn seat_view(board: &FloatingBoard, seat: Seat) -> View<Pixels> {
    let extent = board.workspace_bounds();
    let (mx, my) = marker_position(&seat, extent.size.x, extent.size.y);
    let marker = Vec2::<Pixels>::new(extent.min.x + mx, extent.min.y + my);
    // Center just a step inside the marker: the card lane sits near
    // the frame's edge, with the table clipping off the overhang.
    let center = Vec2::<Pixels>::new(
        marker.x + (extent.center().x - marker.x) * 0.15,
        marker.y + (extent.center().y - marker.y) * 0.15,
    );
    let size = Vec2::<Pixels>::new(extent.size.x * 0.4, extent.size.y * 0.4);
    let framed = Rect::centered(center, size);
    let region = framed.intersection(&extent).unwrap_or(framed);
    View {
        region,
        rotation: board.view().rotation,
    }
}

/// Draws one seat marker: ring, gaze indicator and the seat name.
/// `Anchored`/`Virtual` draw solid, `Absent` dims - the marker is
/// gone, the claim is not.
fn draw_seat(
    cr: &gtk4::cairo::Context,
    seat: &TableTopSeat,
    index: usize,
    width: f64,
    height: f64,
) {
    let (x, y) = marker_position(&seat.pose.seat, width, height);
    let (r, g, b) = match seat.marker {
        Some(_) => SEAT_COLORS[index % SEAT_COLORS.len()],
        None => (0.55, 0.57, 0.60),
    };
    let alpha = if seat.state == SeatMarkerState::Absent {
        0.3
    } else {
        1.0
    };

    cr.set_source_rgba(r, g, b, 0.25 * alpha);
    cr.arc(x, y, MARKER_RADIUS, 0.0, std::f64::consts::TAU);
    let _ = cr.fill();
    cr.set_source_rgba(r, g, b, alpha);
    cr.set_line_width(2.0);
    cr.new_sub_path();
    cr.arc(x, y, MARKER_RADIUS, 0.0, std::f64::consts::TAU);
    let _ = cr.stroke();

    // Gaze indicator: the seat's free radial orientation - drawn in
    // window space, which is the surface frame for this demo.
    let orientation = seat.pose.orientation.as_f64();
    cr.move_to(x, y);
    cr.line_to(
        x + orientation.cos() * GAZE_LENGTH,
        y + orientation.sin() * GAZE_LENGTH,
    );
    let _ = cr.stroke();

    // Seat name, tucked toward the window's inside.
    let (tx, ty) = match seat.pose.seat.side {
        TableSide::Bottom => (x, y - MARKER_RADIUS - 8.0),
        TableSide::Left => (x + MARKER_RADIUS + 8.0, y + 4.0),
        TableSide::Top => (x, y + MARKER_RADIUS + 16.0),
        TableSide::Right => (x - MARKER_RADIUS - 8.0, y + 4.0),
    };
    let name = seat.name.to_string();
    if let Ok(extents) = cr.text_extents(&name) {
        cr.move_to(tx - extents.width() / 2.0, ty);
        let _ = cr.show_text(&name);
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // GTK runs the glib main loop on this thread; the daemon client's
    // websocket tasks need a tokio reactor, hosted on its own runtime.
    let runtime = Rc::new(Runtime::new().expect("tokio runtime"));
    let daemon = runtime.block_on(DaemonSession::connect());

    let application = gtk4::Application::new(
        Some("io.smearor.tangible-objects.seats"),
        Default::default(),
    );
    application.connect_activate(move |app| build_ui(app, daemon.clone(), Rc::clone(&runtime)));
    application.run();
}

/// The running daemon session: the client plus the seat roster stream
/// the GTK side attaches to.
struct DaemonSession {
    /// Shared client - the event pump and the shutdown cleanup use it.
    client: Arc<GraphQLClient>,
    /// Roster updates pushed by the event pump (one full snapshot per
    /// relevant event - simple, cheap, always consistent).
    roster_rx: async_channel::Receiver<Vec<TableTopSeat>>,
}

impl DaemonSession {
    /// Connects to the daemon, claims the session and fills it with
    /// the four virtual seats. `None` when the daemon is unreachable -
    /// the demo still runs its board on the virtual roster.
    async fn connect() -> Option<Rc<Self>> {
        let client = match GraphQLClient::new(Endpoint::resolve(None)) {
            Ok(client) => Arc::new(client),
            Err(error) => {
                warn!("daemon client unavailable: {error} - running without seat events");
                return None;
            }
        };
        let owner = Owner::new(OWNER);
        let session = SeatSessionName::new(SESSION);

        match client.claim_seat_session(session.clone(), &owner).await {
            Ok(ClaimSeatSessionResult::Claimed(_)) => info!("claimed seat session '{SESSION}'"),
            Ok(ClaimSeatSessionResult::Rejected(rejection)) => {
                warn!("seat session rejected: {rejection:?}")
            }
            Err(error) => {
                warn!("daemon unreachable: {error} - running without seat events");
                return None;
            }
        }

        for (side, gaze) in VIRTUAL_SEATS {
            let pose = SeatPose::new(Seat::at(side, Fraction::HALF), RotationRad::new(gaze));
            match client
                .add_seat(
                    session.clone(),
                    SeatName::new(format!("seat-{side}")),
                    None,
                    Some(pose),
                    &owner,
                )
                .await
            {
                Ok(AddSeatResult::Added(added)) => info!("added virtual seat {}", added.seat.name),
                Ok(AddSeatResult::Rejected(rejection)) => warn!("add_seat rejected: {rejection:?}"),
                Err(error) => warn!("add_seat failed: {error}"),
            }
        }

        // A tangible only spawns while its profile is enabled - a
        // disabled profile is invisible to tracking, so no
        // `TANGIBLE_ADDED` would ever arrive for it. The demo claims
        // and enables every profile so any dropped marker can anchor a
        // seat; `unregister_owner` reverts the activation on quit.
        match client.profiles(&ProfileQuery::default()).await {
            Ok(profiles) => {
                for profile in profiles {
                    match client.claim_profile(profile.pattern_id, &owner, true).await {
                        Ok(ClaimProfileResult::Claimed(_)) => {}
                        Ok(ClaimProfileResult::Rejected(rejection)) => {
                            warn!("profile {} rejected: {rejection:?}", profile.name)
                        }
                        Err(error) => warn!("claim_profile failed: {error}"),
                    }
                }
            }
            Err(error) => warn!("profiles query failed: {error}"),
        }

        // Tangibles already resting on the table fired their
        // `TANGIBLE_ADDED` before the stream exists - adopt them the
        // same way.
        if let Ok(tangibles) = client.tangibles().await {
            for tangible in tangibles {
                adopt_marker(&client, tangible.pattern_id).await;
            }
        }

        let (roster_tx, roster_rx) = async_channel::bounded(8);
        let mut stream = match client
            .event_stream(&EventStreamQuery {
                kinds: vec![
                    StreamEvent::TangibleAdded,
                    StreamEvent::SeatMarkerBound,
                    StreamEvent::SeatMarkerAbsent,
                ],
                ..Default::default()
            })
            .await
        {
            Ok(stream) => stream,
            Err(error) => {
                warn!("event stream unavailable: {error} - running without seat events");
                return Some(Rc::new(Self { client, roster_rx }));
            }
        };

        // Event pump: a spawned task on the client's runtime - it owns
        // all daemon traffic after startup and pushes roster snapshots
        // to the GTK side.
        let pump = Arc::clone(&client);
        tokio::spawn(async move {
            if let Ok(seats) = session_seats(&pump).await {
                let _ = roster_tx.try_send(seats);
            }
            while let Some(item) = stream.next().await {
                match item {
                    Ok(event) => {
                        if event.kind == EventKind::TangibleAdded
                            && let Some(tangible) = event.tangible
                        {
                            adopt_marker(&pump, tangible.pattern_id).await;
                        }
                        if let Ok(seats) = session_seats(&pump).await {
                            let _ = roster_tx.try_send(seats);
                        }
                    }
                    Err(error) => warn!("seat event stream error: {error}"),
                }
            }
        });

        Some(Rc::new(Self { client, roster_rx }))
    }

    /// Drops every registration of this example - the session, its
    /// seats and the claimed marker profiles.
    async fn unregister(&self) {
        if let Err(error) = self.client.unregister_owner(&Owner::new(OWNER)).await {
            warn!("unregister_owner failed: {error}");
        }
    }
}

/// The session's current roster - one `seatSessions` query, mapped to
/// the seats for both the pass roster and the status line.
async fn session_seats(
    client: &GraphQLClient,
) -> Result<Vec<TableTopSeat>, tangible_objects_client::ClientError> {
    Ok(client
        .seat_sessions(Some(&SeatSessionName::new(SESSION)))
        .await?
        .into_iter()
        .flat_map(|session| session.seats)
        .collect())
}

/// A tangible appeared: claim its profile for this example and rebind
/// the first still-virtual seat to the marker (`updateSeat`) - the
/// daemon re-derives the seat's pose from the live marker. A pattern
/// already anchoring a seat is skipped - a re-placed tangible rebinds
/// its existing seat instead of consuming another virtual one.
async fn adopt_marker(client: &GraphQLClient, marker: tangible_objects_client::model::PatternId) {
    let owner = Owner::new(OWNER);
    let Ok(seats) = session_seats(client).await else {
        return;
    };
    if seats.iter().any(|seat| seat.marker == Some(marker)) {
        return;
    }

    let profile_name = match client.claim_profile(marker, &owner, true).await {
        Ok(ClaimProfileResult::Claimed(claimed)) => {
            info!("claimed marker profile {marker} ({})", claimed.name);
            Some(claimed.name.to_string())
        }
        Ok(ClaimProfileResult::Rejected(rejection)) => {
            warn!("marker profile {marker} rejected: {rejection:?}");
            None
        }
        Err(error) => {
            warn!("claim_profile failed: {error}");
            None
        }
    };

    let Some(virtual_seat) = seats.iter().find(|seat| seat.marker.is_none()) else {
        info!("no virtual seat left for marker {marker}");
        return;
    };
    let update = SeatUpdate {
        marker: Some(marker),
        ..Default::default()
    };
    match client.update_seat(virtual_seat.id, update, &owner).await {
        Ok(UpdateSeatResult::Updated(updated)) => {
            info!("seat {} now anchors to marker {marker}", updated.seat.name)
        }
        Ok(UpdateSeatResult::Rejected(rejection)) => warn!("update_seat rejected: {rejection:?}"),
        Err(error) => warn!("update_seat failed: {error}"),
    }

    // Rename the seat after its marker - a duplicate profile name only
    // costs the rename, not the binding above. The profile name shows
    // up in the overlay and the daemon GUI's Seats card.
    if let Some(name) = profile_name {
        let rename = SeatUpdate {
            name: Some(SeatName::new(name)),
            ..Default::default()
        };
        match client.update_seat(virtual_seat.id, rename, &owner).await {
            Ok(UpdateSeatResult::Updated(_)) => {}
            Ok(UpdateSeatResult::Rejected(rejection)) => {
                warn!("seat rename rejected: {rejection:?}")
            }
            Err(error) => warn!("seat rename failed: {error}"),
        }
    }
}

fn build_ui(
    application: &gtk4::Application,
    daemon: Option<Rc<DaemonSession>>,
    runtime: Rc<Runtime>,
) {
    let window = gtk4::ApplicationWindow::new(application);
    window.set_title(Some("Tangible Objects - Table Top Seats"));
    window.set_default_size(960, 640);

    // --- Toolbar ------------------------------------------------------------
    let toolbar = gtk4::Box::new(Orientation::Horizontal, 8);
    toolbar.set_margin_top(6);
    toolbar.set_margin_bottom(6);
    toolbar.set_margin_start(8);
    toolbar.set_margin_end(8);
    let pass_button = gtk4::Button::with_label("Pass ⟳");
    let auto_check = gtk4::CheckButton::with_label("Auto");
    auto_check.set_active(true);
    let status = Label::new(None);
    status.set_hexpand(true);
    status.set_halign(gtk4::Align::End);
    status.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    toolbar.append(&pass_button);
    toolbar.append(&auto_check);
    toolbar.append(&status);

    // --- The board ----------------------------------------------------------
    // The roster: virtual seats until the daemon pushes snapshots.
    let roster: Rc<RefCell<Vec<TableTopSeat>>> = Rc::new(RefCell::new(
        VIRTUAL_SEATS
            .iter()
            .enumerate()
            .map(|(index, (side, gaze))| virtual_seat(index as u32, *side, *gaze))
            .collect(),
    ));

    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_vexpand(true);
    drawing_area.set_hexpand(true);
    {
        let roster = Rc::clone(&roster);
        drawing_area.set_draw_func(move |_, cr, width, height| {
            cr.set_source_rgb(0.16, 0.18, 0.22);
            let _ = cr.paint();

            // Hint, dimmed as part of the background.
            cr.select_font_face(
                "Sans",
                gtk4::cairo::FontSlant::Normal,
                gtk4::cairo::FontWeight::Normal,
            );
            cr.set_font_size(13.0);
            let hint = "Drop a marker tangible on the table to take over a virtual seat.";
            if let Ok(extents) = cr.text_extents(hint) {
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.35);
                cr.move_to(
                    (f64::from(width) - extents.width()) / 2.0 - extents.x_bearing(),
                    f64::from(height) / 2.0 - extents.y_bearing(),
                );
                let _ = cr.show_text(hint);
            }

            cr.select_font_face(
                "Sans",
                gtk4::cairo::FontSlant::Normal,
                gtk4::cairo::FontWeight::Bold,
            );
            cr.set_font_size(12.0);
            for (index, seat) in roster.borrow().iter().enumerate() {
                draw_seat(cr, seat, index, f64::from(width), f64::from(height));
            }
        });
    }

    let content = gtk4::Box::new(Orientation::Vertical, 0);
    content.append(&drawing_area);
    let board = FloatingBoard::new(&content);
    board.set_vexpand(true);
    // Seats are table sides, not screen sides: under `SeatAnchor::View`
    // the pass lane is derived from the view *before* `goto_view` flies,
    // so the card anchors outside the region the camera lands on.
    board.set_seat_anchor(floating_board::SeatAnchor::Workspace);

    // Card content: the demo label plus a badge naming the seat the
    // card currently belongs to - updated on `card-seat-changed`.
    let body = gtk4::Box::new(Orientation::Vertical, 4);
    body.append(&Label::new(Some("Pass me around!")));
    let badge = Label::new(None);
    body.append(&badge);
    let card_id = board
        .add_card_with_id(
            "passed-card",
            &body,
            CardConfig {
                title: Some("Passed card".to_string()),
                ..Default::default()
            },
        )
        .unwrap_or_else(|_| floating_board::CardId::new("passed-card"));

    if let Ok(card) = board.card(&card_id) {
        let badge_roster = Rc::clone(&roster);
        badge.set_label(&seat_label(card.seat(), &badge_roster.borrow()));
        card.connect_closure(
            "card-seat-changed",
            false,
            glib::closure_local!(move |card: floating_board::Card| {
                badge.set_label(&seat_label(card.seat(), &badge_roster.borrow()));
            }),
        );
    }

    let pass = {
        let board = board.clone();
        let card_id = card_id.clone();
        let roster = Rc::clone(&roster);
        move || {
            let seats: Vec<Seat> = roster.borrow().iter().map(|seat| seat.pose.seat).collect();
            if let Err(error) =
                board.rotate_to_next_occupied_seat(&card_id, &seats, TurnDirection::Clockwise, true)
            {
                warn!("pass failed: {error}");
                return;
            }
            // Prezi flight onto the seat corner, not the card:
            // `Glide` flies the camera straight to the seat framing -
            // `Transit` would cross the pull-back overview first.
            if let Some(seat) = board.card_seat(&card_id) {
                let target = seat_view(&board, seat);
                info!(
                    "pass: seat {seat:?} target {:?} from {:?}",
                    target,
                    board.view()
                );
                board.goto_view(
                    target,
                    GotoTransition::Glide,
                    Duration::from_millis(GOTO_FLIGHT_MS),
                );
            }
        }
    };

    pass_button.connect_clicked({
        let pass = pass.clone();
        move |_| pass()
    });
    glib::timeout_add_local(Duration::from_millis(3500), move || {
        if auto_check.is_active() {
            pass();
        }
        glib::ControlFlow::Continue
    });

    // Daemon roster updates on the glib loop.
    if let Some(daemon) = daemon.clone() {
        let roster_rx = daemon.roster_rx.clone();
        glib::spawn_future_local(async move {
            while let Ok(seats) = roster_rx.recv().await {
                *roster.borrow_mut() = seats.clone();
                status.set_label(&roster_label(&seats));
                drawing_area.queue_draw();
            }
        });
        // Cleanup on quit: release the session and all profile claims.
        window.connect_close_request(move |_| {
            runtime.block_on(daemon.unregister());
            glib::Propagation::Proceed
        });
    }

    let root = gtk4::Box::new(Orientation::Vertical, 0);
    root.append(&toolbar);
    root.append(&board);
    window.set_child(Some(&root));
    window.present();
}

/// The card badge text for a seat assignment: the roster entry's
/// name when the `Seat` matches (profile names after marker
/// adoption), a `side@offset` fallback when the roster moved on.
fn seat_label(seat: Option<Seat>, roster: &[TableTopSeat]) -> String {
    let Some(seat) = seat else {
        return "Seat: -".to_string();
    };
    match roster.iter().find(|entry| entry.pose.seat == seat) {
        Some(entry) => format!("Seat: {}", entry.name),
        None => {
            let offset = seat
                .offset
                .map(|offset| format!("{:.2}", offset.as_f64()))
                .unwrap_or_else(|| "-".to_string());
            format!("Seat: {}@{offset}", seat.side)
        }
    }
}

/// One status line per seat: name, side/fraction and its anchor state.
fn roster_label(seats: &[TableTopSeat]) -> String {
    seats
        .iter()
        .map(|seat| {
            let side = format!("{}", seat.pose.seat.side);
            let offset = seat
                .pose
                .seat
                .offset
                .map(|offset| format!("{:.2}", offset.as_f64()))
                .unwrap_or_else(|| "-".to_string());
            let state = match seat.state {
                SeatMarkerState::Anchored => "anchored",
                SeatMarkerState::Absent => "absent",
                SeatMarkerState::Virtual => "virtual",
            };
            format!("{}:{side}@{offset} ({state})", seat.name)
        })
        .collect::<Vec<_>>()
        .join("  ·  ")
}
