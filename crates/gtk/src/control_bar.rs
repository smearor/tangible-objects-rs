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
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::Vec2;

use crate::evdev_thread::STABLE_THRESHOLD;
use crate::gui_mode::GuiMode;
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

    // Mode toggle.
    let mode_btn = Button::with_label("Modus: Live");
    mode_btn.add_css_class("suggested-action");

    // Name entry.
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some("Figurenname..."));
    name_entry.set_width_chars(20);
    name_entry.set_sensitive(false);

    // Save button.
    let save_btn = Button::with_label("Profil speichern");
    save_btn.set_sensitive(false);

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

        mode_btn_clone.set_label(if is_training {
            "Modus: Training"
        } else {
            "Modus: Live"
        });
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
            Ok(()) => eprintln!("Profil gespeichert: {}", filepath.display()),
            Err(e) => eprintln!("Fehler beim Speichern: {e}"),
        }
    });

    bar.append(&mode_btn);
    bar.append(&name_entry);
    bar.append(&save_btn);
    bar.append(&status);

    // Status update inside the redraw timer.
    let state_status = state.clone();
    let status_clone = status.clone();
    let save_btn_status = save_btn.clone();
    let da = drawing_area.clone();
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_status.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        let count = s.touches.len();
        match s.mode {
            GuiMode::Live => {
                let n = s.matches.len();
                status_clone.set_label(&format!(
                    "{} Figur(en) erkannt | {} Touch-Punkt(e)",
                    n, count
                ));
            }
            GuiMode::Training => {
                if let Some(err) = &s.device_error {
                    status_clone.set_label(err);
                } else if count < 3 {
                    status_clone.set_label(&format!("Warte auf ≥3 Punkte... ({})", count));
                } else if s.stable {
                    status_clone.set_label("STABIL - Name eingeben und Speichern klicken");
                } else {
                    status_clone.set_label(&format!(
                        "Stabilisiere... {}/{}",
                        s.stable_frames, STABLE_THRESHOLD
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
