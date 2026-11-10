//! The touches card - live raw contacts (slot, tracking id, surface
//! position) as a movable card on the `FloatingBoard`.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use gtk4::Box as GtkBox;
use gtk4::Grid;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;

use crate::language_loader;
use crate::shared_state::SharedState;

/// Widgets of one contact row - the row exists while the contact
/// (slot + tracking id) is live; only the position label rewrites.
struct TouchRow {
    /// Multitouch slot the contact occupies.
    slot: u32,
    /// Kernel tracking id of the contact lifetime.
    tracking_id: i32,
    /// Current surface position cell.
    position: Label,
}

/// Builds the touches-card content: `slot | tracking id | x,y mm`
/// rows rebuilt whenever the contact set changes (a slot that swaps
/// tracking ids is a fresh contact), positions updated every 100 ms.
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
        header.set_halign(gtk4::Align::Start);
        header.set_opacity(0.6);
        grid.attach(&header, col as i32, 0, 1, 1);
    }

    let rows: Rc<RefCell<Vec<TouchRow>>> = Rc::new(RefCell::new(Vec::new()));
    let empty_shown = Rc::new(std::cell::Cell::new(false));
    let state_tick = state.clone();
    let rows_tick = rows.clone();
    let grid_tick = grid.clone();
    let empty_label = fl!(language_loader(), "touches-empty");
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_tick.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        let mut rows = rows_tick.borrow_mut();
        let contacts: Vec<_> = s
            .touches
            .iter()
            .map(|t| (t.slot.as_u32(), t.tracking_id.as_i32()))
            .collect();
        let stale = rows.len() != contacts.len()
            || rows
                .iter()
                .zip(contacts.iter())
                .any(|(row, (slot, id))| row.slot != *slot || row.tracking_id != *id);
        if stale {
            // Rebuild rows 1..: contact rows plus a possibly visible
            // "no contacts" label (new, ended or re-tracked contact).
            let below_header = rows.len() + usize::from(empty_shown.get());
            for _ in 0..below_header {
                grid_tick.remove_row(1);
            }
            rows.clear();
            empty_shown.set(false);
            if contacts.is_empty() {
                let label = Label::new(Some(&empty_label));
                label.set_halign(gtk4::Align::Start);
                label.set_opacity(0.5);
                grid_tick.attach(&label, 0, 1, 3, 1);
                empty_shown.set(true);
            } else {
                for (row_index, (slot, tracking_id)) in contacts.iter().enumerate() {
                    let slot_label = Label::new(Some(&slot.to_string()));
                    slot_label.set_halign(gtk4::Align::Start);
                    let id_label = Label::new(Some(&tracking_id.to_string()));
                    id_label.set_halign(gtk4::Align::Start);
                    let position = Label::new(Some("–"));
                    position.set_halign(gtk4::Align::End);
                    let grid_row = (row_index + 1) as i32;
                    grid_tick.attach(&slot_label, 0, grid_row, 1, 1);
                    grid_tick.attach(&id_label, 1, grid_row, 1, 1);
                    grid_tick.attach(&position, 2, grid_row, 1, 1);
                    rows.push(TouchRow {
                        slot: *slot,
                        tracking_id: *tracking_id,
                        position,
                    });
                }
            }
        }
        for (row, touch) in rows.iter().zip(s.touches.iter()) {
            row.position
                .set_label(&format!("{:.0},{:.0} mm", touch.pos.x, touch.pos.y));
        }
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 0);
    body.append(&grid);
    body
}
