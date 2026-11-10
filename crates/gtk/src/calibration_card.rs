//! The calibration card - the active surface calibration and where it
//! came from (device firmware vs device profile) as a movable card on
//! the `FloatingBoard`.

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

use crate::calibration_source::CalibrationSource;
use crate::language_loader;
use crate::shared_state::SharedState;

/// Builds the calibration-card content: a `label | value` grid
/// refreshed from `SharedState` every 100 ms - the calibration lands
/// once the evdev loop applied the effective calibration.
pub fn create_calibration_card(state: &Arc<Mutex<SharedState>>) -> GtkBox {
    let grid = Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(4);

    let source_value = add_row(&grid, 0, &fl!(language_loader(), "cal-source"));
    let surface_value = add_row(&grid, 1, &fl!(language_loader(), "cal-surface"));
    let x_range_value = add_row(&grid, 2, &fl!(language_loader(), "device-x-range"));
    let y_range_value = add_row(&grid, 3, &fl!(language_loader(), "device-y-range"));
    let res_value = add_row(&grid, 4, &fl!(language_loader(), "device-resolution"));

    let state_tick = state.clone();
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_tick.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        source_value.set_label(&match s.calibration_source {
            Some(CalibrationSource::Device) => {
                fl!(language_loader(), "cal-source-device")
            }
            Some(CalibrationSource::Profile) => {
                fl!(language_loader(), "cal-source-profile")
            }
            None => "–".to_string(),
        });
        match &s.calibration {
            Some(cal) => {
                surface_value.set_label(&format!(
                    "{:.0} × {:.0} mm",
                    cal.width_mm().as_f64(),
                    cal.height_mm().as_f64()
                ));
                x_range_value.set_label(&format!(
                    "{:.0} – {:.0}",
                    cal.x_min.as_f64(),
                    cal.x_max.as_f64()
                ));
                y_range_value.set_label(&format!(
                    "{:.0} – {:.0}",
                    cal.y_min.as_f64(),
                    cal.y_max.as_f64()
                ));
                res_value.set_label(&format!(
                    "{:.2} × {:.2}/mm",
                    cal.x_res.as_f64(),
                    cal.y_res.as_f64()
                ));
            }
            None => {
                for value in [&surface_value, &x_range_value, &y_range_value, &res_value] {
                    value.set_label("–");
                }
            }
        }
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 0);
    body.append(&grid);
    body
}

/// Appends a `label | value` row to `grid` at `row`; returns the
/// value label so the refresh tick can rewrite it.
fn add_row(grid: &Grid, row: i32, name: &str) -> Label {
    let name_label = Label::new(Some(name));
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_opacity(0.6);
    let value = Label::new(Some("–"));
    value.set_halign(gtk4::Align::End);
    grid.attach(&name_label, 0, row, 1, 1);
    grid.attach(&value, 1, row, 1, 1);
    value
}
