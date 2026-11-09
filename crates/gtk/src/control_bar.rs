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
        let count = s.touches.len();
        match s.mode {
            GuiMode::Live => {
                let n = s.matches.len();
                status_clone.set_label(&fl!(
                    language_loader(),
                    "live-status",
                    figures = (n as i64),
                    touches = (count as i64)
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

    bar
}
