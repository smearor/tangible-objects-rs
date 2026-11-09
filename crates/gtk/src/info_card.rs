//! The info card - live counters (touch points, figures, fields) as a
//! movable, rotatable and scalable card on the `FloatingBoard`
//! (concepts/planned/GUI.md §3).

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use gtk4::Box as GtkBox;
use gtk4::Button;
use gtk4::Grid;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;

use crate::evdev_thread::STABLE_THRESHOLD;
use crate::gui_mode::GuiMode;
use crate::language_loader;
use crate::shared_state::SharedState;

/// Builds the info-card content widget: a `label | value` grid the
/// update timer refreshes from [`SharedState`] every 100 ms - the same
/// cadence the control bar's status label uses.
pub fn create_info_card(state: &Arc<Mutex<SharedState>>) -> GtkBox {
    let grid = Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(4);

    // Row 0: the Live/Training mode button - it fills the row in
    // place of a `Mode | value` label pair (moved from the control
    // bar with the card migration).
    let mode_btn = Button::with_label(&fl!(language_loader(), "mode-live"));
    mode_btn.add_css_class("suggested-action");
    let state_mode = state.clone();
    mode_btn.connect_clicked(move |b| {
        let label = match state_mode.lock() {
            Ok(mut s) => {
                s.mode = if s.mode == GuiMode::Live {
                    GuiMode::Training
                } else {
                    GuiMode::Live
                };
                s.stable_frames = 0;
                s.stable = false;
                match s.mode {
                    GuiMode::Live => fl!(language_loader(), "mode-live"),
                    GuiMode::Training => fl!(language_loader(), "mode-training"),
                }
            }
            Err(_) => return,
        };
        b.set_label(&label);
    });
    grid.attach(&mode_btn, 0, 0, 2, 1);

    let touches_value = add_row(&grid, 1, &fl!(language_loader(), "info-touches"));
    let figures_value = add_row(&grid, 2, &fl!(language_loader(), "info-figures"));
    let tracked_value = add_row(&grid, 3, &fl!(language_loader(), "info-tracked"));
    let fields_value = add_row(&grid, 4, &fl!(language_loader(), "info-fields"));
    let zones_value = add_row(&grid, 5, &fl!(language_loader(), "info-dead-zones"));
    let filtered_value = add_row(&grid, 6, &fl!(language_loader(), "info-filtered"));
    let stability_value = add_row(&grid, 7, &fl!(language_loader(), "info-stability"));

    let state_tick = state.clone();
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_tick.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        mode_btn.set_label(&match s.mode {
            GuiMode::Live => fl!(language_loader(), "mode-live"),
            GuiMode::Training => fl!(language_loader(), "mode-training"),
        });
        touches_value.set_label(&s.touches.len().to_string());
        figures_value.set_label(&s.matches.len().to_string());
        tracked_value.set_label(&s.tracked.len().to_string());
        fields_value.set_label(&fl!(
            language_loader(),
            "info-fields-value",
            occupied = (i64::from(s.occupied_fields.as_u32())),
            total = (i64::from(s.field_count.as_u32())),
            sets = (i64::from(s.field_set_count.as_u32()))
        ));
        zones_value.set_label(&fl!(
            language_loader(),
            "info-zones-value",
            zones = (s.dead_zones.len() as i64),
            filtered = (i64::from(s.dead_zone_filtered.as_u32()))
        ));
        filtered_value.set_label(&fl!(
            language_loader(),
            "info-filtered-value",
            suppressed = (i64::from(
                s.edge_filtered.as_u32()
                    + s.dead_zone_filtered.as_u32()
                    + s.mode_suppressed.as_u32()
            )),
            forwarded = (i64::from(s.forwarded.as_u32()))
        ));
        stability_value.set_label(&if s.stable {
            fl!(language_loader(), "info-stable")
        } else {
            fl!(
                language_loader(),
                "info-stabilizing",
                stable = (s.stable_frames as i64),
                threshold = (STABLE_THRESHOLD as i64)
            )
        });
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 0);
    body.append(&grid);
    body
}

/// Appends a `label | value` row to `grid` and returns the value label -
/// the row name is rendered dimmed in column 0, the live value in
/// column 1.
fn add_row(grid: &Grid, row: i32, label: &str) -> Label {
    let name = Label::new(Some(label));
    name.set_halign(gtk4::Align::Start);
    name.add_css_class("dim-label");
    grid.attach(&name, 0, row, 1, 1);

    let value = Label::new(Some("–"));
    value.set_halign(gtk4::Align::Start);
    grid.attach(&value, 1, row, 1, 1);
    value
}
