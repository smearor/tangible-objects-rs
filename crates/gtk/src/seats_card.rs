//! The seats card - every claimed seat session's roster (seat name,
//! owner, marker profile, table side, edge fraction, radial
//! orientation) as a movable card on the `FloatingBoard`
//! (TANGIBLE_OBJECTS_TABLE_TOP_SEAT.md).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use gtk4::Align;
use gtk4::Box as GtkBox;
use gtk4::Grid;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use smearor_commons_layout::TableSide;
use tangible_objects_model::SeatId;
use tangible_objects_model::SeatMarkerState;

use crate::language_loader;
use crate::shared_state::SharedState;

/// Widgets of one seat row - kept across ticks so only the volatile
/// cells (pose, marker state) rewrite.
struct SeatRow {
    /// The seat this row displays - the rebuild key.
    id: SeatId,
    /// Seat name cell - dimmed while the marker is absent.
    name: Label,
    /// Session owner cell.
    owner: Label,
    /// Marker profile name cell - the localized `seat-virtual`
    /// placeholder for virtual seats.
    profile: Label,
    /// Organizational edge cell (`Bottom`/`Left`/`Top`/`Right`).
    side: Label,
    /// Fraction along the edge cell - `"–"` for unpinned offsets.
    fraction: Label,
    /// Radial orientation cell, degrees.
    rotation: Label,
}

/// Builds the seats-card content: a `Seat name | Owner | Profile name |
/// Table side | Fraction | Rotation` grid refreshed from
/// [`SharedState`] every 100 ms. Rows rebuild only when the seat set
/// changes; the pose cells update in place.
pub fn create_seats_card(state: &Arc<Mutex<SharedState>>) -> GtkBox {
    let grid = Grid::new();
    grid.set_column_spacing(10);
    grid.set_row_spacing(4);
    for (column, header) in [
        fl!(language_loader(), "col-seat-name"),
        fl!(language_loader(), "col-owner"),
        fl!(language_loader(), "col-profile-name"),
        fl!(language_loader(), "col-side"),
        fl!(language_loader(), "col-fraction"),
        fl!(language_loader(), "col-rotation"),
    ]
    .iter()
    .enumerate()
    {
        let label = Label::new(Some(header));
        label.set_halign(Align::Start);
        label.add_css_class("dim-label");
        grid.attach(&label, column as i32, 0, 1, 1);
    }

    let rows: Rc<RefCell<Vec<SeatRow>>> = Rc::new(RefCell::new(Vec::new()));
    let state_tick = state.clone();
    let rows_tick = rows.clone();
    let grid_tick = grid.clone();
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let grid = &grid_tick;
        let s = match state_tick.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        // Session order is stable; each session's roster keeps its
        // declaration order - the flattened list is the card's row set.
        let seats = s
            .seat_sessions
            .iter()
            .flat_map(|session| session.seats.iter())
            .collect::<Vec<_>>();
        let mut rows = rows_tick.borrow_mut();
        if rows
            .iter()
            .map(|row| row.id)
            .ne(seats.iter().map(|seat| seat.id))
        {
            // Seat set changed - drop every data row below the header
            // and rebuild in roster order.
            let row_count = rows.len() as i32;
            rows.clear();
            for _ in 0..row_count {
                grid.remove_row(1);
            }
            for (index, seat) in seats.iter().enumerate() {
                let name = Label::new(None);
                let owner = Label::new(None);
                let profile = Label::new(None);
                let side = Label::new(None);
                let fraction = Label::new(None);
                let rotation = Label::new(None);
                let row = index as i32 + 1;
                for (column, label) in [&name, &owner, &profile, &side, &fraction, &rotation]
                    .iter()
                    .enumerate()
                {
                    label.set_halign(Align::Start);
                    grid.attach(*label, column as i32, row, 1, 1);
                }
                rows.push(SeatRow {
                    id: seat.id,
                    name,
                    owner,
                    profile,
                    side,
                    fraction,
                    rotation,
                });
            }
        }
        for (seat, row) in seats.iter().zip(rows.iter()) {
            row.name.set_label(seat.name.as_str());
            // Absent markers dim the whole row - mirrors the seat
            // overlay's alpha reduction while the marker is lifted.
            let absent = seat.state == SeatMarkerState::Absent;
            for label in [
                &row.name,
                &row.owner,
                &row.profile,
                &row.side,
                &row.fraction,
                &row.rotation,
            ] {
                if absent {
                    label.add_css_class("dim-label");
                } else {
                    label.remove_css_class("dim-label");
                }
            }
            row.owner.set_label(seat.owner.as_str());
            let profile = seat
                .marker
                .map(|marker| {
                    s.profiles
                        .iter()
                        .find(|profile| profile.pattern_id == marker)
                        .map(|profile| profile.name.to_string())
                        .unwrap_or_else(|| marker.to_string())
                })
                .unwrap_or_else(|| fl!(language_loader(), "seat-virtual"));
            row.profile.set_label(&profile);
            row.side.set_label(&match seat.pose.seat.side {
                TableSide::Bottom => fl!(language_loader(), "side-bottom"),
                TableSide::Left => fl!(language_loader(), "side-left"),
                TableSide::Top => fl!(language_loader(), "side-top"),
                TableSide::Right => fl!(language_loader(), "side-right"),
            });
            row.fraction.set_label(
                &seat
                    .pose
                    .seat
                    .offset
                    .map(|offset| format!("{:.2}", offset.as_f64()))
                    .unwrap_or_else(|| "–".to_string()),
            );
            row.rotation
                .set_label(&format!("{:.0}°", seat.pose.orientation.to_degrees()));
        }
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 6);
    body.append(&grid);
    body
}
