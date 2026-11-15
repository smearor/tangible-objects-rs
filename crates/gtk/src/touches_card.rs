//! The touches card - live raw contacts (slot, tracking id, surface
//! position) as a movable card on the `FloatingBoard`.

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

use crate::language_loader;
use crate::shared_state::SharedState;

/// Rows are reserved in chunks of this size and never removed once
/// created - the card always keeps room for the highest chunk it has
/// seen, so contacts coming and going do not resize it.
const TOUCH_ROW_CHUNK: usize = 5;

/// Widgets of one contact row - rows persist (blanked while unused),
/// so the grid keeps a stable size while contacts come and go.
struct TouchRow {
    /// Multitouch slot cell.
    slot: Label,
    /// Kernel tracking id cell.
    tracking_id: Label,
    /// Current surface position cell.
    position: Label,
}

/// Builds the touches-card content: `slot | tracking id | x,y mm`
/// rows on a persistent grid sized in `TOUCH_ROW_CHUNK` chunks (grown
/// ahead, never shrunk) so the card does not resize as contacts come
/// and go; cell contents updated every 100 ms.
pub fn create_touches_card(state: &Arc<Mutex<SharedState>>) -> GtkBox {
    let grid = Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(4);

    let headers = [
        fl!(language_loader(), "touch-slot"),
        fl!(language_loader(), "touch-id"),
        fl!(language_loader(), "touch-pos"),
    ];
    for (col, label) in headers.iter().enumerate() {
        let header = Label::new(Some(label));
        header.set_halign(Align::Start);
        header.set_opacity(0.6);
        grid.attach(&header, col as i32, 0, 1, 1);
    }

    // Invisible sizing labels keep the columns from collapsing when
    // few contacts (or none) are live - they measure like real
    // content but never render.
    for (col, sample) in ["99", "9999999", "9999,9999 mm"].iter().enumerate() {
        let ghost = Label::new(Some(sample));
        ghost.set_halign(Align::Start);
        ghost.set_opacity(0.0);
        grid.attach(&ghost, col as i32, 1, 1, 1);
    }

    let empty_label = Label::new(Some(&fl!(language_loader(), "touches-empty")));
    empty_label.set_halign(Align::Start);
    empty_label.set_opacity(0.5);
    empty_label.set_visible(false);
    grid.attach(&empty_label, 0, 1, 3, 1);

    let rows: Rc<RefCell<Vec<TouchRow>>> = Rc::new(RefCell::new(Vec::new()));
    let state_tick = state.clone();
    let rows_tick = rows.clone();
    let grid_tick = grid.clone();
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_tick.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        let mut rows = rows_tick.borrow_mut();
        // One chunk is always reserved ahead: reaching the current
        // capacity appends another chunk instead of resizing the card
        // for every new contact.
        let needed = TOUCH_ROW_CHUNK * (s.touches.len() / TOUCH_ROW_CHUNK + 1);
        while rows.len() < needed {
            let slot = Label::new(Some(" "));
            slot.set_halign(Align::Start);
            let tracking_id = Label::new(Some(" "));
            tracking_id.set_halign(Align::Start);
            let position = Label::new(Some(" "));
            position.set_halign(Align::End);
            let grid_row = (rows.len() + 1) as i32;
            grid_tick.attach(&slot, 0, grid_row, 1, 1);
            grid_tick.attach(&tracking_id, 1, grid_row, 1, 1);
            grid_tick.attach(&position, 2, grid_row, 1, 1);
            rows.push(TouchRow {
                slot,
                tracking_id,
                position,
            });
        }
        empty_label.set_visible(s.touches.is_empty());
        for (index, row) in rows.iter().enumerate() {
            match s.touches.get(index) {
                Some(touch) => {
                    row.slot.set_label(&touch.slot.as_u32().to_string());
                    row.tracking_id
                        .set_label(&touch.tracking_id.as_i32().to_string());
                    row.position
                        .set_label(&format!("{:.0},{:.0} mm", touch.pos.x, touch.pos.y));
                }
                None => {
                    row.slot.set_label(" ");
                    row.tracking_id.set_label(" ");
                    row.position.set_label(" ");
                }
            }
        }
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 0);
    body.append(&grid);
    body
}
