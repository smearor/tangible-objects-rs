//! The device card - input-device identity and raw axis ranges as a
//! movable card on the `FloatingBoard` (concepts/planned/GUI.md §3).

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
use tangible_objects_model::DeviceInfo;

use crate::language_loader;
use crate::shared_state::SharedState;

/// Builds the device-card content: a `label | value` grid refreshed
/// from `SharedState` every 100 ms - the device info lands once the
/// evdev loop has reported it, `device_error` covers the open-failure
/// path (the same string the canvas would show).
pub fn create_device_card(state: &Arc<Mutex<SharedState>>, device_path: &str) -> GtkBox {
    let grid = Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(4);

    let name_value = add_row(&grid, 0, &fl!(language_loader(), "device-name"));
    let path_value = add_row(&grid, 1, &fl!(language_loader(), "device-path"));
    let surface_value = add_row(&grid, 2, &fl!(language_loader(), "device-surface"));
    let res_value = add_row(&grid, 3, &fl!(language_loader(), "device-resolution"));
    let x_range_value = add_row(&grid, 4, &fl!(language_loader(), "device-x-range"));
    let y_range_value = add_row(&grid, 5, &fl!(language_loader(), "device-y-range"));
    let status_value = add_row(&grid, 6, &fl!(language_loader(), "device-status"));

    path_value.set_label(device_path);

    let state_tick = state.clone();
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_tick.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        match &s.device_info {
            Some(info) => {
                surface_value.set_label(&match &s.calibration {
                    Some(cal) => format!(
                        "{:.0} × {:.0} mm",
                        cal.width_mm().as_f64(),
                        cal.height_mm().as_f64()
                    ),
                    None => "–".to_string(),
                });
                fill_device_rows(
                    &name_value,
                    &res_value,
                    &x_range_value,
                    &y_range_value,
                    info,
                );
            }
            None => {
                for value in [
                    &name_value,
                    &surface_value,
                    &res_value,
                    &x_range_value,
                    &y_range_value,
                ] {
                    value.set_label("–");
                }
            }
        }
        match &s.device_error {
            Some(error) => status_value.set_label(error),
            None => status_value.set_label(&fl!(language_loader(), "device-status-ok")),
        }
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 0);
    body.append(&grid);
    body
}

/// Writes the [`DeviceInfo`] rows - ranges show the raw axis span
/// (`min–max`), resolution the units-per-mm conversion factor.
fn fill_device_rows(
    name: &Label,
    resolution: &Label,
    x_range: &Label,
    y_range: &Label,
    info: &DeviceInfo,
) {
    name.set_label(info.name.as_deref().unwrap_or("–"));
    resolution.set_label(&match (info.x_res, info.y_res) {
        (Some(x), Some(y)) => format!("{:.2} × {:.2}/mm", x.as_f64(), y.as_f64()),
        _ => "–".to_string(),
    });
    x_range.set_label(&range_label(info.x_min, info.x_max));
    y_range.set_label(&range_label(info.y_min, info.y_max));
}

/// The `min–max` label for a raw axis range, `–` when unknown.
fn range_label(
    min: Option<tangible_objects_model::RawAxisValue>,
    max: Option<tangible_objects_model::RawAxisValue>,
) -> String {
    match (min, max) {
        (Some(min), Some(max)) => format!("{:.0} – {:.0}", min.as_f64(), max.as_f64()),
        _ => "–".to_string(),
    }
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
