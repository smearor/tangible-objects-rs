//! The field-sets card - the registered field sets (name, owner,
//! enabled state, field count) as a movable card on the
//! `FloatingBoard` (concepts/planned/GUI.md §3).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use gtk4::Box as GtkBox;
use gtk4::Grid;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::Switch;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use tangible_objects_model::FieldSetName;
use tangible_objects_runtime::ControlCommand;

use crate::language_loader;
use crate::shared_state::SharedState;
use crate::shared_state::send_command;

/// Widgets of one field-set row - kept across ticks so only the
/// volatile cells rewrite.
struct FieldSetRowWidgets {
    /// The set this row displays.
    name: FieldSetName,
    /// Registering application cell.
    owner: Label,
    /// Enable switch - sends `SetFieldSetEnabled`.
    enabled: Switch,
    /// Occupancy field count cell.
    field_count: Label,
}

/// Builds the field-sets-card content: a `Name | Owner | Enabled |
/// Fields` grid refreshed from [`SharedState`] every 100 ms. Rows
/// rebuild only when the registered sets change; the volatile cells
/// (enabled flag) update in place.
pub fn create_field_sets_card(state: &Arc<Mutex<SharedState>>) -> GtkBox {
    let grid = Grid::new();
    grid.set_column_spacing(10);
    grid.set_row_spacing(4);
    for (column, header) in [
        fl!(language_loader(), "col-name"),
        fl!(language_loader(), "col-owner"),
        fl!(language_loader(), "col-enabled"),
        fl!(language_loader(), "col-fields"),
    ]
    .iter()
    .enumerate()
    {
        let label = Label::new(Some(header));
        label.set_halign(gtk4::Align::Start);
        label.add_css_class("dim-label");
        grid.attach(&label, column as i32, 0, 1, 1);
    }

    let rows: Rc<RefCell<Vec<FieldSetRowWidgets>>> = Rc::new(RefCell::new(Vec::new()));
    let state_tick = state.clone();
    let rows_tick = rows.clone();
    let grid_tick = grid.clone();
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let grid = &grid_tick;
        let s = match state_tick.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        let mut rows = rows_tick.borrow_mut();
        if rows
            .iter()
            .map(|row| row.name.as_str())
            .ne(s.field_set_rows.iter().map(|row| row.name.as_str()))
        {
            // Registration set changed - drop every data row below the
            // header and rebuild in `s.field_set_rows` order.
            let row_count = rows.len() as i32;
            rows.clear();
            for _ in 0..row_count {
                grid.remove_row(1);
            }
            for (index, set) in s.field_set_rows.iter().enumerate() {
                let name = Label::new(Some(set.name.as_str()));
                name.set_halign(gtk4::Align::Start);
                let owner = Label::new(None);
                owner.set_halign(gtk4::Align::Start);
                let enabled = Switch::new();
                enabled.set_halign(gtk4::Align::Start);
                let set_name = set.name.clone();
                let state_enabled = state_tick.clone();
                enabled.connect_state_set(move |_, active| {
                    send_command(
                        &state_enabled,
                        ControlCommand::SetFieldSetEnabled {
                            name: set_name.clone(),
                            enabled: active,
                        },
                    );
                    gtk4::glib::Propagation::Proceed
                });
                let field_count = Label::new(None);
                field_count.set_halign(gtk4::Align::Start);
                let row = index as i32 + 1;
                grid.attach(&name, 0, row, 1, 1);
                grid.attach(&owner, 1, row, 1, 1);
                grid.attach(&enabled, 2, row, 1, 1);
                grid.attach(&field_count, 3, row, 1, 1);
                rows.push(FieldSetRowWidgets {
                    name: set.name.clone(),
                    owner,
                    enabled,
                    field_count,
                });
            }
        }
        for (set, row) in s.field_set_rows.iter().zip(rows.iter()) {
            row.owner.set_label(
                set.owner
                    .as_ref()
                    .map(|owner| owner.as_str())
                    .unwrap_or("–"),
            );
            // set_active emits no state-set signal - the sync does
            // not echo commands back to the device loop.
            row.enabled.set_active(set.enabled);
            row.field_count.set_label(&set.field_count.to_string());
        }
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 0);
    body.append(&grid);
    body
}
