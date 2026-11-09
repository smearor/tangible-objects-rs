//! The bottom control bar - mode toggle, name entry, save button,
//! status label.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use gtk4::Align;
use gtk4::Box as GtkBox;
use gtk4::Button;
use gtk4::DrawingArea;
use gtk4::Entry;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::Vec2;
use tracing::error;
use tracing::info;

use crate::evdev_thread::STABLE_THRESHOLD;
use crate::gui_mode::GuiMode;
use crate::language_loader;
use crate::shared_state::SharedState;

/// Tolerance the saved profile takes (scalar mm).
const DEFAULT_TOLERANCE: PatternTolerance = PatternTolerance::new(3.0);

/// Builds the control bar widget and wires its callbacks.
pub fn create_control_bar(
    state: &Arc<Mutex<SharedState>>,
    profiles_dir: &Option<String>,
    drawing_area: &DrawingArea,
) -> GtkBox {
    let bar = GtkBox::new(Orientation::Horizontal, 8);
    bar.set_valign(Align::End);
    bar.set_halign(Align::Fill);
    bar.set_margin_bottom(8);
    bar.set_margin_start(12);
    bar.set_margin_end(12);
    bar.set_spacing(8);

    // Name entry.
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some(&fl!(language_loader(), "figure-name-placeholder")));
    name_entry.set_width_chars(20);
    name_entry.set_sensitive(false);

    // Save button.
    let save_btn = Button::with_label(&fl!(language_loader(), "save-profile"));
    save_btn.set_sensitive(false);

    // Status label.
    let status = Label::new(Some(""));
    status.set_hexpand(true);
    status.set_halign(Align::End);

    // Save callback.
    let state_save = state.clone();
    let name_entry_save = name_entry.clone();
    let profiles_dir_save = profiles_dir.clone();
    save_btn.connect_clicked(move |_| {
        let name: String = name_entry_save.text().to_string();
        if name.is_empty() {
            return;
        }

        let points: Vec<Vec2<Millimeters>> = match state_save.lock() {
            Ok(s) => {
                if !s.stable {
                    return;
                }
                s.touches.iter().map(|t| t.pos).collect()
            }
            Err(_) => return,
        };

        if points.len() < 3 {
            return;
        }

        let profile = ProfileFile::from_points(&name, &points, DEFAULT_TOLERANCE);
        let dir = profiles_dir_save.as_deref().unwrap_or("profiles");
        std::fs::create_dir_all(dir).ok();
        let filename = format!("{}/{}.toml", dir, name.to_lowercase().replace(' ', "-"));
        let filepath = PathBuf::from(&filename);

        match profile.save(&filepath) {
            Ok(()) => info!(path = %filepath.display(), "profile saved"),
            Err(e) => error!(error = %e, "profile save failed"),
        }
    });

    bar.append(&name_entry);
    bar.append(&save_btn);
    bar.append(&status);

    // Status update inside the redraw timer.
    let state_status = state.clone();
    let status_clone = status.clone();
    let save_btn_status = save_btn.clone();
    let name_entry_status = name_entry.clone();
    let da = drawing_area.clone();
    let mut applied_mode = state_status.lock().map(|s| s.mode).unwrap_or(GuiMode::Live);
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_status.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        // The profile-capture entry follows the mode wherever it is
        // toggled (info card button now owns the switch).
        if s.mode != applied_mode {
            applied_mode = s.mode;
            name_entry_status.set_sensitive(applied_mode == GuiMode::Training);
        }
        let count = s.touches.len();
        match s.mode {
            GuiMode::Live => {
                let n = s.matches.len();
                status_clone.set_label(&fl!(
                    language_loader(),
                    "live-status",
                    figures = (n as i64),
                    touches = (count as i64),
                    occupied = (i64::from(s.occupied_fields.as_u32())),
                    fields = (i64::from(s.field_count.as_u32())),
                    sets = (i64::from(s.field_set_count.as_u32())),
                    zones = (s.dead_zones.len() as i64),
                    filtered = (i64::from(s.dead_zone_filtered.as_u32())),
                    suppressed = (i64::from(
                        s.edge_filtered.as_u32()
                            + s.dead_zone_filtered.as_u32()
                            + s.mode_suppressed.as_u32()
                    )),
                    forwarded = (i64::from(s.forwarded.as_u32()))
                ));
            }
            GuiMode::Training => {
                if let Some(err) = &s.device_error {
                    status_clone.set_label(err);
                } else if count < 3 {
                    status_clone.set_label(&fl!(
                        language_loader(),
                        "wait-for-points",
                        count = (count as i64)
                    ));
                } else if s.stable {
                    status_clone.set_label(&fl!(language_loader(), "stable-hint"));
                } else {
                    status_clone.set_label(&fl!(
                        language_loader(),
                        "stabilizing",
                        stable = (s.stable_frames as i64),
                        threshold = (STABLE_THRESHOLD as i64)
                    ));
                }
                save_btn_status.set_sensitive(s.stable);
            }
        }
        drop(s);
        da.queue_draw();
        ControlFlow::Continue
    });

    let outer = GtkBox::new(Orientation::Vertical, 4);
    outer.set_valign(Align::End);
    outer.append(&bar);

    outer
}

/// Wires the touch-passthrough input-region sync.
///
/// While enabled, the surface accepts input only on the control bar and
/// the board's cards - every other touch falls through to the desktop
/// beneath (Wayland `set_input_region`; a no-op elsewhere). The region is
/// rebuilt on every tick: card footprints move, rotate and scale with the
/// gestures. Called once from `run_gui` after the `FloatingBoard` exists.
pub fn wire_passthrough(
    outer: &GtkBox,
    board: &floating_board::FloatingBoard,
    state: &Arc<Mutex<SharedState>>,
) {
    let outer_sync = outer.clone();
    let board_sync = board.clone();
    let state_passthrough = state.clone();
    let mut applied = false;
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        // Read the flag under lock, drop it before the GTK surface
        // calls - `set_input_region` may dispatch into event handlers
        // that take `state` themselves.
        let enabled = state_passthrough
            .lock()
            .map(|s| s.touch_passthrough)
            .unwrap_or_default();
        if enabled {
            // The board must not reset the region on its next
            // allocation - the consumer owns it while passthrough runs.
            board_sync.set_input_region_managed(true);
            if let Some(region) = passthrough_region(&outer_sync, &board_sync)
                && let Some(surface) = outer_sync.native().and_then(|n| n.surface())
            {
                surface.set_input_region(Some(&region));
                applied = true;
            }
        } else if applied {
            applied = false;
            board_sync.set_input_region_managed(false);
            if let Some(surface) = outer_sync.native().and_then(|n| n.surface()) {
                surface.set_input_region(None);
            }
        }
        ControlFlow::Continue
    });
}

/// The passthrough input region in surface coordinates: the control-bar
/// rectangle plus the rotated footprints of every card on the board.
fn passthrough_region(
    outer: &GtkBox,
    board: &floating_board::FloatingBoard,
) -> Option<gtk4::cairo::Region> {
    let native = outer.native()?;
    let (x, y, width, height) = outer_surface_rect(outer);
    if width <= 0 || height <= 0 {
        return None;
    }
    let region =
        gtk4::cairo::Region::create_rectangle(&gtk4::cairo::RectangleInt::new(x, y, width, height));
    let cards = board.cards_region();
    if !cards.is_empty()
        && let Some(origin) = board.compute_point(&native, &gtk4::graphene::Point::new(0.0, 0.0))
    {
        cards.translate(origin.x().round() as i32, origin.y().round() as i32);
        let _ = region.union(&cards);
    }
    Some(region)
}

/// The widget's allocation rectangle in surface coordinates - the
/// input-region rect of the control-bar box.
fn outer_surface_rect(widget: &impl gtk4::prelude::WidgetExt) -> (i32, i32, i32, i32) {
    let Some(native) = widget.native() else {
        return (0, 0, widget.width(), widget.height());
    };
    let origin = widget
        .compute_point(&native, &gtk4::graphene::Point::new(0.0, 0.0))
        .map(|p| (p.x().round() as i32, p.y().round() as i32))
        .unwrap_or_default();
    (origin.0, origin.1, widget.width(), widget.height())
}
