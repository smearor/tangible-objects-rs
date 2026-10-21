//! Cairo drawing - the surface scene per mode (live figures /
//! training capture) plus the frame chrome.

use std::sync::Arc;
use std::sync::Mutex;

use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::Pixels;
use tangible_objects_model::TouchSet;
use tangible_objects_model::TrackedSet;
use tangible_objects_model::Vec2;

use crate::evdev_thread::STABLE_THRESHOLD;
use crate::gui_mode::GuiMode;
use crate::shared_state::SharedState;

/// Per-figure accent color - stable hash over the pattern ID.
fn figure_color(id: PatternId) -> (f64, f64, f64) {
    match id.as_u32() % 6 {
        0 => (0.2, 0.6, 1.0),
        1 => (0.2, 0.9, 0.3),
        2 => (1.0, 0.5, 0.2),
        3 => (1.0, 0.3, 0.5),
        4 => (0.8, 0.3, 1.0),
        _ => (1.0, 0.9, 0.2),
    }
}

/// Draws the full frame: background, overlays, mode scene.
pub fn draw_scene(cr: &cairo::Context, width: i32, height: i32, state: &Arc<Mutex<SharedState>>) {
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

    // User-facing message pushed via an interface (ShowMessage).
    if let Some(msg) = &s.message {
        cr.set_source_rgb(1.0, 0.9, 0.2);
        cr.set_font_size(42.0);
        if let Ok(ext) = cr.text_extents(msg) {
            cr.move_to((w - ext.width()) / 2.0, 64.0);
        } else {
            cr.move_to(20.0, 64.0);
        }
        let _ = cr.show_text(msg);
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
        draw_training(cr, w, h, touches.into(), &to_screen, &s);
    } else {
        draw_live(
            cr,
            w,
            h,
            touches.into(),
            matches,
            (&s.tracked).into(),
            &to_screen,
        );
    }
}

/// Training view: complete-graph point set + stability status.
fn draw_training(
    cr: &cairo::Context,
    _w: f64,
    _h: f64,
    touches: TouchSet<'_>,
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

/// Live view: matched figures with names + unmatched points dimmed.
fn draw_live(
    cr: &cairo::Context,
    _w: f64,
    _h: f64,
    touches: TouchSet<'_>,
    matches: &[PatternMatch],
    tracked: TrackedSet<'_>,
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
        // Pending (unconfirmed partial spawn) - dim the whole figure.
        let pending = tracked.iter().any(|t| {
            t.pending
                && t.pattern_id == m.pattern_id
                && t.center.distance(&m.center).as_f64() < 50.0
        });
        let alpha = if pending { 0.25 } else { 1.0 };
        let n = m.touch_indices.len();
        if n < 2 {
            continue;
        }

        // Complete graph.
        cr.set_source_rgba(r, g, b, 0.5 * alpha);
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
            cr.set_source_rgba(r, g, b, alpha);
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
        cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        cr.set_font_size(22.0);
        if let Ok(extents) = cr.text_extents(m.pattern_name.as_str()) {
            let tx = cx - extents.width() / 2.0;
            let ty = (top_y - 28.0).max(extents.height() + 4.0);
            cr.move_to(tx, ty);
            let _ = cr.show_text(m.pattern_name.as_str());
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
