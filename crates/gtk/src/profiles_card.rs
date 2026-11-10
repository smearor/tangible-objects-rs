//! The profiles card - the loaded figure profiles (name, owner,
//! enabled state, current surface position) as a movable card on the
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
use tangible_objects_model::FigureColor;
use tangible_objects_model::PatternId;
use tangible_objects_runtime::ControlCommand;

use crate::language_loader;
use crate::shared_state::SharedState;
use crate::shared_state::send_command;

/// Widgets of one profile row - kept across ticks so only the
/// volatile cells rewrite.
struct ProfileRow {
    /// The profile this row displays.
    id: PatternId,
    /// Registering application cell.
    owner: Label,
    /// Enable switch - sends `SetProfileEnabled`.
    enabled: Switch,
    /// Color preview + picker - sends `SetProfileColor`.
    color: gtk4::ColorDialogButton,
    /// Current surface position cell.
    position: Label,
}

/// Builds the profiles-card content: a `Name | Owner | Enabled |
/// Position` grid refreshed from [`SharedState`] every 100 ms. Rows
/// rebuild only when the registered set changes; the volatile cells
/// (enabled flag, live position) update in place.
pub fn create_profiles_card(state: &Arc<Mutex<SharedState>>) -> GtkBox {
    let grid = Grid::new();
    grid.set_column_spacing(10);
    grid.set_row_spacing(4);
    for (column, header) in [
        fl!(language_loader(), "col-name"),
        fl!(language_loader(), "col-owner"),
        fl!(language_loader(), "col-enabled"),
        fl!(language_loader(), "col-color"),
        fl!(language_loader(), "col-position"),
    ]
    .iter()
    .enumerate()
    {
        let label = Label::new(Some(header));
        label.set_halign(gtk4::Align::Start);
        label.add_css_class("dim-label");
        grid.attach(&label, column as i32, 0, 1, 1);
    }

    let rows: Rc<RefCell<Vec<ProfileRow>>> = Rc::new(RefCell::new(Vec::new()));
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
            .map(|row| row.id)
            .ne(s.profiles.iter().map(|profile| profile.pattern_id))
        {
            // Registration set changed - drop every data row below the
            // header and rebuild in `s.profiles` order.
            let row_count = rows.len() as i32;
            rows.clear();
            for _ in 0..row_count {
                grid.remove_row(1);
            }
            for (index, profile) in s.profiles.iter().enumerate() {
                let name = Label::new(Some(profile.name.as_str()));
                name.set_halign(gtk4::Align::Start);
                let owner = Label::new(None);
                owner.set_halign(gtk4::Align::Start);
                let enabled = Switch::new();
                enabled.set_halign(gtk4::Align::Start);
                let pattern_id = profile.pattern_id;
                let state_enabled = state_tick.clone();
                enabled.connect_state_set(move |_, active| {
                    send_command(
                        &state_enabled,
                        ControlCommand::SetProfileEnabled {
                            id: pattern_id,
                            enabled: active,
                        },
                    );
                    gtk4::glib::Propagation::Proceed
                });
                let color = gtk4::ColorDialogButton::new(Some(gtk4::ColorDialog::new()));
                let (r, g, b) = crate::draw::figure_color(profile.pattern_id);
                color.set_rgba(&gtk4::gdk::RGBA::new(r as f32, g as f32, b as f32, 1.0));
                color.set_halign(gtk4::Align::Start);
                let pattern_id_color = profile.pattern_id;
                let state_color = state_tick.clone();
                color.connect_notify_local(Some("rgba"), move |button, _| {
                    let rgba = button.rgba();
                    send_command(
                        &state_color,
                        ControlCommand::SetProfileColor {
                            id: pattern_id_color,
                            color: FigureColor::new(
                                rgba.red() as f64,
                                rgba.green() as f64,
                                rgba.blue() as f64,
                            ),
                        },
                    );
                });
                let position = Label::new(None);
                position.set_halign(gtk4::Align::Start);
                let row = index as i32 + 1;
                grid.attach(&name, 0, row, 1, 1);
                grid.attach(&owner, 1, row, 1, 1);
                grid.attach(&enabled, 2, row, 1, 1);
                grid.attach(&color, 3, row, 1, 1);
                grid.attach(&position, 4, row, 1, 1);
                rows.push(ProfileRow {
                    id: profile.pattern_id,
                    owner,
                    enabled,
                    color,
                    position,
                });
            }
        }
        for (profile, row) in s.profiles.iter().zip(rows.iter()) {
            // Reflect the live color (declared `color=` or a
            // `SetProfileColor` from elsewhere).
            let (r, g, b) = s
                .pattern_styles
                .iter()
                .find(|style| style.pattern_id == profile.pattern_id)
                .and_then(|style| style.color)
                .map(|color| color.rgb())
                .unwrap_or_else(|| crate::draw::figure_color(profile.pattern_id));
            let current = row.color.rgba();
            let epsilon = 1.0 / 255.0;
            if (current.red() as f64 - r).abs() > epsilon
                || (current.green() as f64 - g).abs() > epsilon
                || (current.blue() as f64 - b).abs() > epsilon
            {
                row.color
                    .set_rgba(&gtk4::gdk::RGBA::new(r as f32, g as f32, b as f32, 1.0));
            }
            row.owner.set_label(
                profile
                    .owner
                    .as_ref()
                    .map(|owner| owner.as_str())
                    .unwrap_or("–"),
            );
            // set_active emits no state-set signal - the sync does
            // not echo commands back to the device loop.
            row.enabled.set_active(profile.enabled);
            let position = s
                .tracked
                .iter()
                .find(|tangible| tangible.pattern_id == profile.pattern_id)
                .map(|tangible| format!("{:.0},{:.0} mm", tangible.center.x, tangible.center.y))
                .unwrap_or_else(|| "–".to_string());
            row.position.set_label(&position);
        }
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 0);
    body.append(&grid);
    body
}
