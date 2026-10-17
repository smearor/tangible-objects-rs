//! Fullscreen GTK4/Cairo visualization (temporary home in the daemon until
//! the dedicated `tangible-objects-ui` crate exists).

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use gtk4::Align;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::Box as GtkBox;
use gtk4::Button;
use gtk4::DrawingArea;
use gtk4::Entry;
use gtk4::EventControllerKey;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::Overlay;
use gtk4::gdk;
use gtk4::glib;
use gtk4::glib::ControlFlow;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use tangible_objects_engine::Engine;
use tangible_objects_engine::TouchDevice;
use tangible_objects_engine::TouchState;
use tangible_objects_model::Calibration;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::Pixels;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::TangiblePattern;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::Vec2;

const STABLE_THRESHOLD: u32 = 10;
const DEFAULT_TOLERANCE: PatternTolerance = PatternTolerance::new(3.0);

#[derive(Clone, Copy, PartialEq)]
enum GuiMode {
    Live,
    Training,
}

struct SharedState {
    touches: Vec<TouchPoint>,
    matches: Vec<PatternMatch>,
    calibration: Option<Calibration>,
    stable_frames: u32,
    stable: bool,
    mode: GuiMode,
    device_error: Option<String>,
}

/// Starts the fullscreen GUI on the given evdev device path.
pub fn run_gui(device_path: &str, profiles_dir: Option<&str>, edge: Option<Millimeters>) {
    let app = Application::builder()
        .application_id("org.tangibleobjects.gui")
        .build();

    let patterns = crate::modes::load_patterns(profiles_dir);
    let device_path = device_path.to_string();
    let profiles_dir = profiles_dir.map(|s| s.to_string());

    app.connect_activate(move |app| {
        let win = ApplicationWindow::builder()
            .application(app)
            .title("Tangible Objects")
            .build();
        win.fullscreen();

        let state = Arc::new(Mutex::new(SharedState {
            touches: Vec::new(),
            matches: Vec::new(),
            calibration: None,
            stable_frames: 0,
            stable: false,
            mode: GuiMode::Live,
            device_error: None,
        }));

        // Background thread reading evdev events.
        let state_thread = state.clone();
        let patterns_thread = patterns.clone();
        let dev_path_thread = device_path.clone();
        std::thread::spawn(move || {
            evdev_thread(dev_path_thread, patterns_thread, state_thread, edge);
        });

        // DrawingArea.
        let drawing_area = DrawingArea::new();
        let state_draw = state.clone();
        drawing_area.set_draw_func(move |_, cr, w, h| {
            draw_scene(cr, w, h, &state_draw);
        });

        // Control bar at the bottom.
        let controls = create_control_bar(&state, &profiles_dir, &drawing_area);

        // Overlay: DrawingArea + control bar.
        let overlay = Overlay::new();
        overlay.set_child(Some(&drawing_area));
        overlay.add_overlay(&controls);

        win.set_child(Some(&overlay));

        // Escape/Q to quit.
        let key_ctrl = EventControllerKey::new();
        let win_clone = win.clone();
        key_ctrl.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape || key == gdk::Key::q {
                win_clone.close();
            }
            Propagation::Proceed
        });
        win.add_controller(key_ctrl);

        win.present();

        // Redraw timer (30 fps).
        let da = drawing_area.clone();
        glib::timeout_add_local(Duration::from_millis(33), move || {
            da.queue_draw();
            ControlFlow::Continue
        });
    });

    // GTK parses argv for its own options — the daemon's mode flag
    // (`--gui`) is not one of them, so run with an empty argument list.
    app.run_with_args(&[] as &[String]);
}

fn mode_is_live(state: &Arc<Mutex<SharedState>>) -> bool {
    state
        .lock()
        .map(|s| s.mode == GuiMode::Live)
        .unwrap_or(false)
}

fn evdev_thread(
    device_path: String,
    patterns: Vec<TangiblePattern>,
    state: Arc<Mutex<SharedState>>,
    edge: Option<Millimeters>,
) {
    let mut device = match TouchDevice::open(std::path::Path::new(&device_path)) {
        Ok(d) => d,
        Err(e) => {
            if let Ok(mut s) = state.lock() {
                s.device_error = Some(format!("Gerät konnte nicht geöffnet werden: {e}"));
            }
            return;
        }
    };

    let mut engine = Engine::new(patterns);
    if let Some(mm) = edge {
        engine.set_edge_margin(mm);
    }
    let mut touch_state = TouchState::new();
    if let Some(cal) = device.calibration() {
        touch_state.set_calibration(cal);
        engine.set_calibration(cal);
        if let Ok(mut s) = state.lock() {
            s.calibration = Some(cal);
        }
    }

    let mut stable_count: Option<usize> = None;
    let mut stable_frames: u32 = 0;

    while let Ok(events) = device.fetch_events() {
        for raw in events {
            touch_state.process(&raw);

            // The engine builds its own touch state from every event —
            // feeding it only SYN_REPORTs leaves that state empty and no
            // figure ever matches.
            let frame = if mode_is_live(&state) {
                engine.process(&raw)
            } else {
                None
            };

            if raw.is_syn_report() {
                let touches = touch_state.active_touches();
                let count = touches.len();
                let matches = frame.map(|f| f.matches).unwrap_or_default();

                // Stability check (for training mode).
                if stable_count == Some(count) && count >= 3 {
                    stable_frames += 1;
                } else {
                    stable_count = Some(count);
                    stable_frames = 0;
                }

                if let Ok(mut s) = state.lock() {
                    s.touches = touches;
                    s.matches = matches;
                    s.stable_frames = stable_frames;
                    s.stable = stable_frames >= STABLE_THRESHOLD && count >= 3;
                }
            }
        }
    }
}

fn create_control_bar(
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
        let filepath = std::path::PathBuf::from(&filename);

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
    glib::timeout_add_local(Duration::from_millis(100), move || {
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

fn figure_color(id: tangible_objects_model::PatternId) -> (f64, f64, f64) {
    match id.as_u32() % 6 {
        0 => (0.2, 0.6, 1.0),
        1 => (0.2, 0.9, 0.3),
        2 => (1.0, 0.5, 0.2),
        3 => (1.0, 0.3, 0.5),
        4 => (0.8, 0.3, 1.0),
        _ => (1.0, 0.9, 0.2),
    }
}

fn draw_scene(cr: &cairo::Context, width: i32, height: i32, state: &Arc<Mutex<SharedState>>) {
    let w = width as f64;
    let h = height as f64;

    // Black background.
    cr.set_source_rgb(0.05, 0.05, 0.08);
    let _ = cr.paint();

    let s = match state.lock() {
        Ok(s) => s,
        Err(_) => return,
    };

    // Show device error.
    if let Some(err) = &s.device_error {
        cr.set_source_rgb(1.0, 0.3, 0.3);
        cr.set_font_size(24.0);
        cr.move_to(20.0, 40.0);
        let _ = cr.show_text(err);
        return;
    }

    let cal = match s.calibration {
        Some(c) => c,
        None => {
            cr.set_source_rgb(0.8, 0.8, 0.8);
            cr.set_font_size(20.0);
            cr.move_to(20.0, 40.0);
            let _ = cr
                .show_text("Keine Kalibrierung verfügbar - GUI ohne Auflösungsinfo nicht nutzbar");
            return;
        }
    };

    let mode = s.mode;
    let touches = &s.touches;
    let matches = &s.matches;

    // Map touch points from mm to screen pixels.
    let to_screen = |pos: &Vec2<Millimeters>| -> Vec2<Pixels> {
        cal.mm_to_screen(*pos, Pixels::new(w), Pixels::new(h))
    };

    if mode == GuiMode::Training {
        draw_training(cr, w, h, touches, &to_screen, &s);
    } else {
        draw_live(cr, w, h, touches, matches, &to_screen);
    }
}

fn draw_training(
    cr: &cairo::Context,
    _w: f64,
    _h: f64,
    touches: &[TouchPoint],
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
    s: &SharedState,
) {
    let count = touches.len();
    if count == 0 {
        cr.set_source_rgb(0.6, 0.6, 0.6);
        cr.set_font_size(22.0);
        cr.move_to(20.0, 40.0);
        let _ = cr.show_text("Lege die Figur auf die Touchfläche...");
        return;
    }

    // Draw all pairwise connections (complete graph).
    let line_alpha = if s.stable { 0.6 } else { 0.25 };
    cr.set_line_width(2.0);
    cr.set_source_rgba(0.4, 0.7, 1.0, line_alpha);
    for i in 0..count {
        for j in (i + 1)..count {
            let p1 = to_screen(&touches[i].pos);
            let p2 = to_screen(&touches[j].pos);
            cr.move_to(p1.x, p1.y);
            cr.line_to(p2.x, p2.y);
            let _ = cr.stroke();
        }
    }

    // Touch points as circles.
    let point_color = if s.stable {
        (0.3, 0.9, 0.4)
    } else {
        (0.5, 0.7, 1.0)
    };
    for tp in touches {
        let p = to_screen(&tp.pos);
        cr.set_source_rgb(point_color.0, point_color.1, point_color.2);
        cr.arc(p.x, p.y, 12.0, 0.0, 2.0 * std::f64::consts::PI);
        let _ = cr.fill();

        // Ring.
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.set_line_width(2.0);
        cr.arc(p.x, p.y, 12.0, 0.0, 2.0 * std::f64::consts::PI);
        let _ = cr.stroke();
    }

    // Status text top left.
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.set_font_size(20.0);
    cr.move_to(20.0, 30.0);
    let status_text = if count < 3 {
        format!("Warte auf ≥3 Punkte... ({})", count)
    } else if s.stable {
        format!("STABIL: {} Punkte - Name eingeben und Speichern", count)
    } else {
        format!(
            "Stabilisiere... {}/{} ({} Punkte)",
            s.stable_frames, STABLE_THRESHOLD, count
        )
    };
    let _ = cr.show_text(&status_text);

    // Coordinates next to the points.
    cr.set_font_size(12.0);
    cr.set_source_rgb(0.7, 0.7, 0.7);
    for (i, tp) in touches.iter().enumerate() {
        let p = to_screen(&tp.pos);
        cr.move_to(p.x + 16.0, p.y - 12.0);
        let _ = cr.show_text(&format!("P{} ({:.0},{:.0})", i, tp.pos.x, tp.pos.y));
    }
}

fn draw_live(
    cr: &cairo::Context,
    _w: f64,
    _h: f64,
    touches: &[TouchPoint],
    matches: &[PatternMatch],
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
) {
    // Collect matched indices.
    let mut matched_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for m in matches {
        for &idx in &m.touch_indices {
            matched_indices.insert(idx);
        }
    }

    // Draw figures (lines + names).
    for m in matches {
        let (r, g, b) = figure_color(m.pattern_id);
        let n = m.touch_indices.len();
        if n < 2 {
            continue;
        }

        // Complete graph.
        cr.set_source_rgba(r, g, b, 0.5);
        cr.set_line_width(2.5);
        for i in 0..n {
            for j in (i + 1)..n {
                let p1 = to_screen(&touches[m.touch_indices[i]].pos);
                let p2 = to_screen(&touches[m.touch_indices[j]].pos);
                cr.move_to(p1.x, p1.y);
                cr.line_to(p2.x, p2.y);
                let _ = cr.stroke();
            }
        }

        // Touch points.
        for &idx in &m.touch_indices {
            let p = to_screen(&touches[idx].pos);
            cr.set_source_rgb(r, g, b);
            cr.arc(p.x, p.y, 14.0, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.fill();
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.set_line_width(2.0);
            cr.arc(p.x, p.y, 14.0, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.stroke();
        }

        // Name above the figure (outside the object).
        let ys: Vec<f64> = m
            .touch_indices
            .iter()
            .map(|&idx| to_screen(&touches[idx].pos).y)
            .collect();
        let top_y = ys.iter().cloned().fold(f64::INFINITY, f64::min);
        let cx = to_screen(&m.center).x;
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        cr.set_font_size(22.0);
        if let Ok(extents) = cr.text_extents(&m.pattern_name) {
            let tx = cx - extents.width() / 2.0;
            let ty = (top_y - 28.0).max(extents.height() + 4.0);
            cr.move_to(tx, ty);
            let _ = cr.show_text(&m.pattern_name);
        }
    }

    // Unmatched touch points (gray).
    for (i, tp) in touches.iter().enumerate() {
        if matched_indices.contains(&i) {
            continue;
        }
        let p = to_screen(&tp.pos);
        cr.set_source_rgb(0.4, 0.4, 0.4);
        cr.arc(p.x, p.y, 8.0, 0.0, 2.0 * std::f64::consts::PI);
        let _ = cr.fill();
    }

    // Info top left.
    cr.set_source_rgb(0.7, 0.7, 0.7);
    cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(16.0);
    cr.move_to(20.0, 30.0);
    let _ = cr.show_text(&format!(
        "{} Figur(en) | {} Touch-Punkt(e)",
        matches.len(),
        touches.len()
    ));
}
