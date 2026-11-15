//! The dead-zones card - registered dead-zone points (name, position,
//! radius, active hit, remove button) plus an `Add` button that
//! registers a zone at the single active touch point.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use gtk4::Align;
use gtk4::Box as GtkBox;
use gtk4::Button;
use gtk4::Grid;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use smearor_commons_geometry::Millimeters;
use tangible_objects_model::DeadZone;
use tangible_objects_model::DeadZoneName;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Registration;
use tangible_objects_runtime::ControlCommand;
use tokio::sync::oneshot;

use crate::language_loader;
use crate::shared_state::SharedState;
use crate::shared_state::send_command;

/// Suppression radius a GUI-created dead zone takes (mm).
const DEAD_ZONE_RADIUS_MM: f64 = 20.0;

/// Widgets of one dead-zone row - kept across ticks so only the
/// volatile cells rewrite.
struct DeadZoneRow {
    /// The zone this row displays.
    name: DeadZoneName,
    /// Surface position cell.
    position: Label,
    /// Suppression radius cell.
    radius: Label,
    /// Hit indicator - a touch currently inside the zone.
    hit: Label,
}

/// Builds the dead-zones card content: one row per registered zone
/// (`Name | Position | Radius | Hit | Remove`) plus the `Add` button -
/// armed only while exactly one touch is live, registering a
/// `DEAD_ZONE_RADIUS_MM` zone at that contact's position. Adds and
/// removals stay session-scoped; the Save button commits both to
/// `dead_zones.d`.
pub fn create_dead_zones_card(state: &Arc<Mutex<SharedState>>) -> GtkBox {
    let grid = Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(4);

    let headers = [
        fl!(language_loader(), "dead-zone-name"),
        fl!(language_loader(), "dead-zone-position"),
        fl!(language_loader(), "dead-zone-radius"),
        fl!(language_loader(), "dead-zone-hit"),
        fl!(language_loader(), "dead-zone-remove"),
    ];
    for (col, label) in headers.iter().enumerate() {
        let header = Label::new(Some(label));
        header.set_halign(Align::Start);
        header.set_opacity(0.6);
        grid.attach(&header, col as i32, 0, 1, 1);
    }

    // Adds and removals are session-scoped; the Save button commits
    // the whole current set via `PersistDeadZones` (writes store files
    // for live zones, deletes entries for removed ones).
    let save_btn = Button::with_label(&fl!(language_loader(), "dead-zone-save"));
    let state_save = state.clone();
    save_btn.connect_clicked(move |_| {
        send_command(&state_save, ControlCommand::PersistDeadZones);
    });
    let add_btn = Button::with_label(&fl!(language_loader(), "dead-zone-add"));
    let state_add = state.clone();
    add_btn.connect_clicked(move |_| {
        let zone = {
            let Ok(s) = state_add.lock() else {
                return;
            };
            let Some(touch) = single_touch(&s) else {
                return;
            };
            DeadZone::new(
                DeadZoneName::new(next_zone_name(&s)),
                touch.pos,
                Millimeters::new(DEAD_ZONE_RADIUS_MM),
            )
        };
        let (reply, _ignored) = oneshot::channel();
        send_command(
            &state_add,
            ControlCommand::Register {
                registration: Box::new(Registration::DeadZone(zone)),
                owner: None,
                lifetime: Lifetime::Session,
                reply,
            },
        );
    });

    let rows: Rc<RefCell<Vec<DeadZoneRow>>> = Rc::new(RefCell::new(Vec::new()));
    let state_tick = state.clone();
    let rows_tick = rows.clone();
    let grid_tick = grid.clone();
    let add_btn_tick = add_btn.clone();
    let empty_label = fl!(language_loader(), "dead-zones-empty");
    let empty_shown = Rc::new(std::cell::Cell::new(false));
    // Overlay auto-show: opening the card switches the dead-zone
    // overlay on; closing restores the previous state. `auto_vis`:
    // 0 = untouched, 1 = enable sent (pending sync), 2 = confirmed ours.
    let was_open = std::cell::Cell::new(false);
    let auto_vis = std::cell::Cell::new(0u8);
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_tick.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        let open = grid_tick.is_visible();
        if open && !was_open.get() && !s.dead_zones_visible {
            if let Some(handle) = &s.command_handle {
                handle
                    .try_send_command(ControlCommand::SetDeadZonesVisible(true))
                    .ok();
            }
            auto_vis.set(1);
        } else if !open && was_open.get() && auto_vis.get() > 0 {
            if let Some(handle) = &s.command_handle {
                handle
                    .try_send_command(ControlCommand::SetDeadZonesVisible(false))
                    .ok();
            }
            auto_vis.set(0);
        }
        if open && auto_vis.get() == 1 && s.dead_zones_visible {
            auto_vis.set(2);
        } else if auto_vis.get() == 2 && !s.dead_zones_visible {
            // The user switched the overlay off while the card was
            // open - we no longer own it, closing leaves it off anyway.
            auto_vis.set(0);
        }
        was_open.set(open);
        add_btn_tick.set_sensitive(single_touch(&s).is_some());
        let mut rows = rows_tick.borrow_mut();
        let names: Vec<&DeadZoneName> = s.dead_zones.iter().map(|zone| &zone.name).collect();
        let stale = rows.len() != names.len()
            || rows
                .iter()
                .zip(names.iter())
                .any(|(row, name)| row.name != **name);
        if stale {
            let below_header = rows.len() + usize::from(empty_shown.get());
            for _ in 0..below_header {
                grid_tick.remove_row(1);
            }
            rows.clear();
            empty_shown.set(false);
            if names.is_empty() {
                let label = Label::new(Some(&empty_label));
                label.set_halign(Align::Start);
                label.set_opacity(0.5);
                grid_tick.attach(&label, 0, 1, 5, 1);
                empty_shown.set(true);
            } else {
                for (row_index, name) in names.iter().enumerate() {
                    let name_label = Label::new(Some(name.as_str()));
                    name_label.set_halign(Align::Start);
                    let position = Label::new(Some("–"));
                    position.set_halign(Align::Start);
                    let radius = Label::new(Some("–"));
                    radius.set_halign(Align::Start);
                    let hit = Label::new(Some(""));
                    hit.set_halign(Align::Start);
                    let remove = Button::from_icon_name("user-trash-symbolic");
                    let zone_name = (*name).clone();
                    let state_remove = state_tick.clone();
                    remove.connect_clicked(move |_| {
                        send_command(
                            &state_remove,
                            ControlCommand::UnregisterDeadZone {
                                name: zone_name.clone(),
                                // Session-scoped removal - the store
                                // file survives until Save commits it.
                                unpersist: false,
                            },
                        );
                    });
                    let grid_row = (row_index + 1) as i32;
                    grid_tick.attach(&name_label, 0, grid_row, 1, 1);
                    grid_tick.attach(&position, 1, grid_row, 1, 1);
                    grid_tick.attach(&radius, 2, grid_row, 1, 1);
                    grid_tick.attach(&hit, 3, grid_row, 1, 1);
                    grid_tick.attach(&remove, 4, grid_row, 1, 1);
                    rows.push(DeadZoneRow {
                        name: (*name).clone(),
                        position,
                        radius,
                        hit,
                    });
                }
            }
        }
        for (row, zone) in rows.iter().zip(s.dead_zones.iter()) {
            row.position
                .set_label(&format!("{:.0},{:.0} mm", zone.center.x, zone.center.y));
            row.radius
                .set_label(&format!("{:.0} mm", zone.radius.as_f64()));
            row.hit.set_label(
                if s.unfiltered_touches.iter().any(|t| zone.contains(&t.pos)) {
                    "●"
                } else {
                    ""
                },
            );
        }
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 6);
    body.append(&grid);
    let actions = GtkBox::new(Orientation::Horizontal, 6);
    actions.append(&add_btn);
    actions.append(&save_btn);
    body.append(&actions);
    body
}

/// The single live contact when exactly one is active (the `Add`
/// button's precondition).
fn single_touch(s: &SharedState) -> Option<&tangible_objects_model::TouchPoint> {
    match s.touches.len() {
        1 => s.touches.first(),
        _ => None,
    }
}

/// Next free `zone-N` name for a GUI-created dead zone.
fn next_zone_name(s: &SharedState) -> String {
    (1..)
        .map(|index| format!("zone-{index}"))
        .find(|name| !s.dead_zones.iter().any(|zone| zone.name.as_str() == name))
        .unwrap_or_else(|| "zone-gui".to_string())
}
