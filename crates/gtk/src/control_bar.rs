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
use gtk4::ToggleButton;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use tangible_objects_model::FilterMode;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::Vec2;
use tangible_objects_runtime::ControlCommand;
use tracing::error;
use tracing::info;

use crate::evdev_thread::STABLE_THRESHOLD;
use crate::gui_mode::GuiMode;
use crate::language_loader;
use crate::shared_state::SharedState;

/// Tolerance the saved profile takes (scalar mm).
const DEFAULT_TOLERANCE: PatternTolerance = PatternTolerance::new(3.0);

/// `SharedState` flag reader - the overlay toggles sync their
/// checked state from it.
type OverlayRead = fn(&SharedState) -> bool;

/// Builds the control bar widget and wires its callbacks.
pub fn create_control_bar(
    state: &Arc<Mutex<SharedState>>,
    profiles_dir: &Option<String>,
    drawing_area: &DrawingArea,
    bg_provider: &gtk4::CssProvider,
) -> GtkBox {
    let bar = GtkBox::new(Orientation::Horizontal, 8);
    bar.set_valign(Align::End);
    bar.set_halign(Align::Fill);
    bar.set_margin_bottom(8);
    bar.set_margin_start(12);
    bar.set_margin_end(12);
    bar.set_spacing(8);

    // Mode toggle.
    let mode_btn = Button::with_label(&fl!(language_loader(), "mode-live"));
    mode_btn.add_css_class("suggested-action");

    // Name entry.
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some(&fl!(language_loader(), "figure-name-placeholder")));
    name_entry.set_width_chars(20);
    name_entry.set_sensitive(false);

    // Save button.
    let save_btn = Button::with_label(&fl!(language_loader(), "save-profile"));
    save_btn.set_sensitive(false);

    // Alpha toggle - cycles the background opacity presets; the
    // touches/labels/chrome stay opaque, only the canvas fill thins.
    let alpha_btn = Button::with_label(&fl!(
        language_loader(),
        "background-alpha",
        percent = (state
            .lock()
            .map(|s| s.background_alpha.percent() as i64)
            .unwrap_or(100))
    ));
    let state_alpha = state.clone();
    let alpha_btn_clone = alpha_btn.clone();
    let da_alpha = drawing_area.clone();
    let provider_alpha = bg_provider.clone();
    alpha_btn.connect_clicked(move |_| {
        let Ok(mut s) = state_alpha.lock() else {
            return;
        };
        s.background_alpha = s.background_alpha.next();
        s.background_alpha_request = Some(s.background_alpha);
        alpha_btn_clone.set_label(&fl!(
            language_loader(),
            "background-alpha",
            percent = (s.background_alpha.percent() as i64)
        ));
        provider_alpha.load_from_data(&s.background_alpha.surface_bg_css());
        da_alpha.queue_draw();
    });

    // Status label.
    let status = Label::new(Some(""));
    status.set_hexpand(true);
    status.set_halign(Align::End);

    // Mode toggle callback.
    let state_toggle = state.clone();
    let mode_btn_clone = mode_btn.clone();
    let name_entry_clone = name_entry.clone();
    let save_btn_clone = save_btn.clone();
    mode_btn.connect_clicked(move |_| {
        let is_training = match state_toggle.lock() {
            Ok(mut s) => {
                s.mode = if s.mode == GuiMode::Live {
                    GuiMode::Training
                } else {
                    GuiMode::Live
                };
                s.stable_frames = 0;
                s.stable = false;
                s.mode == GuiMode::Training
            }
            Err(_) => return,
        };

        let label = if is_training {
            fl!(language_loader(), "mode-training")
        } else {
            fl!(language_loader(), "mode-live")
        };
        mode_btn_clone.set_label(&label);
        name_entry_clone.set_sensitive(is_training);
        save_btn_clone.set_sensitive(false);
    });

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

    bar.append(&mode_btn);
    bar.append(&name_entry);
    bar.append(&save_btn);
    bar.append(&alpha_btn);
    bar.append(&status);

    // Second row: overlay toggles (send the same ControlCommands the
    // interfaces do) + the FilterMode button set.
    let toggles = GtkBox::new(Orientation::Horizontal, 8);
    toggles.set_halign(Align::End);

    let mut overlay_buttons: Vec<(ToggleButton, OverlayRead)> = Vec::new();
    let mut add_overlay_toggle =
        |label: &str, read: OverlayRead, command: fn(bool) -> ControlCommand| {
            let btn = ToggleButton::with_label(label);
            btn.set_active(state.lock().map(|s| read(&s)).unwrap_or_default());
            let state_click = state.clone();
            btn.connect_toggled(move |b| {
                send_command(&state_click, command(b.is_active()));
            });
            toggles.append(&btn);
            overlay_buttons.push((btn, read));
        };
    add_overlay_toggle(
        &fl!(language_loader(), "toggle-edge-margin"),
        |s| s.edge_margin_visible,
        ControlCommand::SetEdgeMarginVisible,
    );
    add_overlay_toggle(
        &fl!(language_loader(), "toggle-fields"),
        |s| s.fields_visible,
        ControlCommand::SetFieldsVisible,
    );
    add_overlay_toggle(
        &fl!(language_loader(), "toggle-ruler"),
        |s| s.ruler_visible,
        ControlCommand::SetRulerVisible,
    );
    add_overlay_toggle(
        &fl!(language_loader(), "toggle-positions"),
        |s| s.positions_visible,
        ControlCommand::SetPositionsVisible,
    );
    add_overlay_toggle(
        &fl!(language_loader(), "toggle-passthrough"),
        |s| s.touch_passthrough,
        ControlCommand::SetTouchPassthrough,
    );

    // FilterMode button set - one checked at a time, commands flow
    // through the control channel (honored by the proxy in --filter;
    // display state otherwise).
    let current_mode = state.lock().map(|s| s.filter_mode).unwrap_or_default();
    let add_mode_button = |leader: &mut Option<ToggleButton>,
                           buttons: &mut Vec<(ToggleButton, FilterMode)>,
                           mode: FilterMode,
                           label: &str| {
        let btn = ToggleButton::with_label(label);
        if let Some(group) = &*leader {
            btn.set_group(Some(group));
        } else {
            *leader = Some(btn.clone());
        }
        btn.set_active(mode == current_mode);
        let state_mode = state.clone();
        btn.connect_toggled(move |b| {
            if b.is_active() {
                send_command(
                    &state_mode,
                    ControlCommand::SetFilterMode {
                        mode,
                        stability: None,
                        stage: None,
                    },
                );
            }
        });
        toggles.append(&btn);
        buttons.push((btn, mode));
    };
    let mut mode_group: Option<ToggleButton> = None;
    let mut mode_buttons: Vec<(ToggleButton, FilterMode)> = Vec::new();
    add_mode_button(
        &mut mode_group,
        &mut mode_buttons,
        FilterMode::Off,
        &fl!(language_loader(), "filter-mode-off"),
    );
    add_mode_button(
        &mut mode_group,
        &mut mode_buttons,
        FilterMode::Immediate,
        &fl!(language_loader(), "filter-mode-immediate"),
    );
    add_mode_button(
        &mut mode_group,
        &mut mode_buttons,
        FilterMode::Gated,
        &fl!(language_loader(), "filter-mode-gated"),
    );
    add_mode_button(
        &mut mode_group,
        &mut mode_buttons,
        FilterMode::Guarded,
        &fl!(language_loader(), "filter-mode-guarded"),
    );
    add_mode_button(
        &mut mode_group,
        &mut mode_buttons,
        FilterMode::TangiblesOnly,
        &fl!(language_loader(), "filter-mode-tangibles-only"),
    );

    // Status update inside the redraw timer.
    let state_status = state.clone();
    let status_clone = status.clone();
    let save_btn_status = save_btn.clone();
    let alpha_btn_status = alpha_btn.clone();
    let provider_status = bg_provider.clone();
    let da = drawing_area.clone();
    let mut applied_alpha = state_status
        .lock()
        .map(|s| s.background_alpha)
        .unwrap_or_else(|_| crate::BackgroundAlpha::from_percent(100));
    let mut applied_overlays = state_status
        .lock()
        .map(|s| {
            overlay_buttons
                .iter()
                .map(|(_, read)| read(&s))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut applied_mode = current_mode;
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_status.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        // Alpha changes from `SetBackgroundAlpha` commands land in the
        // shared state via the device loop - the provider reload and
        // button label follow here (the toggle applies both itself).
        if s.background_alpha != applied_alpha {
            applied_alpha = s.background_alpha;
            provider_status.load_from_data(&applied_alpha.surface_bg_css());
            alpha_btn_status.set_label(&fl!(
                language_loader(),
                "background-alpha",
                percent = (applied_alpha.percent() as i64)
            ));
        }
        // Overlay/mode changes from interface commands land in shared
        // state via the device loop - sync the buttons (set_active
        // re-fires toggled and re-sends an idempotent command).
        for ((btn, read), applied) in overlay_buttons.iter().zip(applied_overlays.iter_mut()) {
            let current = read(&s);
            if current != *applied {
                *applied = current;
                btn.set_active(current);
            }
        }
        if s.filter_mode != applied_mode {
            applied_mode = s.filter_mode;
            for (btn, mode) in &mode_buttons {
                btn.set_active(*mode == applied_mode);
            }
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
    outer.append(&toggles);
    outer.append(&bar);

    // Touch passthrough: the surface input region is reduced to the
    // control-bar box so touches elsewhere reach the desktop beneath
    // (Wayland `set_input_region`; a no-op elsewhere). Re-applied
    // whenever the box's allocation changes.
    let mut applied_passthrough: Option<(i32, i32, i32, i32)> = None;
    let outer_sync = outer.clone();
    let state_passthrough = state.clone();
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        // Read the flag under lock, drop it before the GTK surface
        // calls - `set_input_region` may dispatch into event handlers
        // that take `state` themselves.
        let enabled = state_passthrough
            .lock()
            .map(|s| s.touch_passthrough)
            .unwrap_or_default();
        if enabled {
            let rect = outer_surface_rect(&outer_sync);
            if applied_passthrough != Some(rect)
                && let Some(surface) = outer_sync.native().and_then(|n| n.surface())
                && rect.2 > 0
                && rect.3 > 0
            {
                applied_passthrough = Some(rect);
                let region = gtk4::cairo::Region::create_rectangle(
                    &gtk4::cairo::RectangleInt::new(rect.0, rect.1, rect.2, rect.3),
                );
                surface.set_input_region(Some(&region));
            }
        } else if applied_passthrough.is_some() {
            applied_passthrough = None;
            if let Some(surface) = outer_sync.native().and_then(|n| n.surface()) {
                surface.set_input_region(None);
            }
        }
        ControlFlow::Continue
    });

    outer
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

/// Sends a `ControlCommand` over the runtime's command channel - the
/// same path the interface servers take. Uses `try_lock`: the
/// status-sync timer calls `set_active` on the toggle buttons while
/// holding `state`, which re-fires `connect_toggled` on this same
/// thread - a blocking `lock()` would deadlock on the non-recursive
/// mutex. Dropping the (idempotent) command on contention is safe.
fn send_command(state: &Arc<Mutex<SharedState>>, command: ControlCommand) {
    if let Ok(s) = state.try_lock()
        && let Some(handle) = &s.command_handle
    {
        handle.try_send_command(command).ok();
    }
}
