//! Cairo drawing - the surface scene per mode (live figures /
//! training capture) plus the frame chrome.

use std::sync::Arc;
use std::sync::Mutex;

use i18n_embed_fl::fl;
use tangible_objects_model::Calibration;
use tangible_objects_model::DeadZone;
use tangible_objects_model::DisplayRotation;
use tangible_objects_model::FieldSet;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::Pixels;
use tangible_objects_model::Region;
use tangible_objects_model::TouchSet;
use tangible_objects_model::TrackedSet;
use tangible_objects_model::Vec2;

use crate::evdev_thread::STABLE_THRESHOLD;
use crate::gui_mode::GuiMode;
use crate::language_loader;
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

    let s = match state.lock() {
        Ok(s) => s,
        Err(_) => return,
    };

    // The canvas background is the drawing area's own CSS
    // `background-color` (`.surface-bg`, alpha via `BackgroundAlpha`)
    // - a cairo fill would sit *above* that layer as an opaque cover,
    // so nothing is painted here; the widget stays transparent and the
    // desktop shows through below 100% alpha.

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
            let _ = cr.show_text(&fl!(language_loader(), "no-calibration"));
            return;
        }
    };

    let mode = s.mode;
    let touches = &s.touches;
    let matches = &s.matches;

    // Map touch points from mm to screen pixels.
    let rotation = s.rotation;
    let to_screen = |pos: &Vec2<Millimeters>| -> Vec2<Pixels> {
        cal.mm_to_screen(*pos, Vec2::new(w, h), rotation)
    };

    if s.dead_zones_visible {
        draw_dead_zones(cr, &s.dead_zones, &to_screen);
    }
    if s.edge_margin_visible {
        draw_edge_margin(cr, s.edge_margin, &cal, &to_screen);
    }
    if s.fields_visible {
        draw_fields(cr, &s.field_sets, &to_screen);
    }
    if s.grid_visible {
        draw_grid(cr, &cal, &to_screen);
    }

    if mode == GuiMode::Training {
        draw_training(cr, w, h, touches.into(), &to_screen, &s);
    } else {
        draw_live(cr, touches.into(), matches, (&s.tracked).into(), &to_screen);
    }

    if s.positions_visible {
        draw_positions(
            cr,
            touches.into(),
            (&s.tracked).into(),
            &s.field_sets,
            &to_screen,
        );
    }
    if s.ruler_visible {
        draw_ruler(cr, w, h, &cal, &to_screen, rotation);
    }
}

/// Dead-zone overlay: a dashed circle and the zone's name on every
/// suppressed spot, drawn beneath the figure/touch layer.
fn draw_dead_zones(
    cr: &cairo::Context,
    zones: &[DeadZone],
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
) {
    for zone in zones {
        let center = to_screen(&zone.center);
        // The mm→px scale may be anisotropic - sample the radius in
        // both axes and take the mean so the circle stays circular.
        let rx = to_screen(&(zone.center + Vec2::new(zone.radius.as_f64(), 0.0)))
            .distance(&center)
            .as_f64();
        let ry = to_screen(&(zone.center + Vec2::new(0.0, zone.radius.as_f64())))
            .distance(&center)
            .as_f64();
        let radius = (rx + ry) / 2.0;

        cr.set_source_rgba(1.0, 0.3, 0.25, 0.10);
        cr.arc(center.x, center.y, radius, 0.0, 2.0 * std::f64::consts::PI);
        let _ = cr.fill();
        cr.set_source_rgba(1.0, 0.3, 0.25, 0.85);
        cr.set_dash(&[6.0, 4.0], 0.0);
        cr.set_line_width(2.0);
        cr.arc(center.x, center.y, radius, 0.0, 2.0 * std::f64::consts::PI);
        let _ = cr.stroke();
        cr.set_dash(&[], 0.0);

        cr.set_font_size(16.0);
        cr.set_source_rgba(1.0, 0.55, 0.5, 0.9);
        let name = zone.name.as_str();
        let label_y = center.y + radius + 18.0;
        match cr.text_extents(name) {
            Ok(ext) => cr.move_to(center.x - ext.width() / 2.0, label_y),
            Err(_) => cr.move_to(center.x, label_y),
        }
        let _ = cr.show_text(name);
    }
}

/// Edge-margin overlay: semi-transparent bands and a dashed inner
/// border marking the strip where contacts are ingest-filtered.
fn draw_edge_margin(
    cr: &cairo::Context,
    margin: Millimeters,
    cal: &Calibration,
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
) {
    let m = margin.as_f64();
    if m <= 0.0 {
        return;
    }
    let w_mm = cal.width_mm().as_f64();
    let h_mm = cal.height_mm().as_f64();
    let origin = to_screen(&Vec2::<Millimeters>::new(0.0, 0.0));
    let extent = to_screen(&Vec2::<Millimeters>::new(w_mm, h_mm));
    let inner_tl = to_screen(&Vec2::<Millimeters>::new(m, m));
    let inner_br = to_screen(&Vec2::<Millimeters>::new(w_mm - m, h_mm - m));

    cr.set_source_rgba(1.0, 0.35, 0.2, 0.10);
    for rect in [
        (origin.x, origin.y, extent.x, inner_tl.y),     // top band
        (origin.x, inner_br.y, extent.x, extent.y),     // bottom band
        (origin.x, inner_tl.y, inner_tl.x, inner_br.y), // left band
        (inner_br.x, inner_tl.y, extent.x, inner_br.y), // right band
    ] {
        if rect.2 > rect.0 && rect.3 > rect.1 {
            cr.rectangle(rect.0, rect.1, rect.2 - rect.0, rect.3 - rect.1);
            let _ = cr.fill();
        }
    }

    cr.set_dash(&[8.0, 5.0], 0.0);
    cr.set_line_width(2.0);
    cr.set_source_rgba(1.0, 0.35, 0.2, 0.65);
    cr.rectangle(
        inner_tl.x,
        inner_tl.y,
        inner_br.x - inner_tl.x,
        inner_br.y - inner_tl.y,
    );
    let _ = cr.stroke();
    cr.set_dash(&[], 0.0);
}

/// Field-set overlay: a dashed, semi-transparent region per field -
/// the occupancy cells the engine counts (suppressed in filter mode).
fn draw_fields(
    cr: &cairo::Context,
    sets: &[FieldSet],
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
) {
    for set in sets {
        // Disabled sets stay registered but inert - drawn faint so
        // the overlay still shows their extent.
        let alpha = if set.enabled { 1.0 } else { 0.3 };
        for field in &set.fields {
            let region = field_screen_region(set, field);
            let tl = to_screen(&Vec2::<Millimeters>::new(
                region.x_min.as_f64(),
                region.y_min.as_f64(),
            ));
            let br = to_screen(&Vec2::<Millimeters>::new(
                region.x_max.as_f64(),
                region.y_max.as_f64(),
            ));

            cr.set_source_rgba(0.2, 0.8, 1.0, 0.08 * alpha);
            cr.rectangle(tl.x, tl.y, br.x - tl.x, br.y - tl.y);
            let _ = cr.fill();
            cr.set_dash(&[6.0, 4.0], 0.0);
            cr.set_line_width(1.5);
            cr.set_source_rgba(0.2, 0.8, 1.0, 0.6 * alpha);
            cr.rectangle(tl.x, tl.y, br.x - tl.x, br.y - tl.y);
            let _ = cr.stroke();
            cr.set_dash(&[], 0.0);

            cr.set_font_size(11.0);
            cr.set_source_rgba(0.55, 0.9, 1.0, 0.85 * alpha);
            cr.move_to(tl.x + 4.0, tl.y + 14.0);
            let _ = cr.show_text(field.name.as_str());
        }
    }
}

/// A field's region resolved into screen-millimeter space - anchored
/// sets carry local/normalized regions relative to their window.
fn field_screen_region(set: &FieldSet, field: &tangible_objects_model::Field) -> Region {
    let Some(anchor) = &set.window else {
        return field.region;
    };
    let width = anchor.width.map(|w| w.as_f64()).unwrap_or(1.0);
    let height = anchor.height.map(|h| h.as_f64()).unwrap_or(1.0);
    let x = anchor.x_offset.as_f64();
    let y = anchor.y_offset.as_f64();
    Region {
        x_min: Millimeters::new(x + field.region.x_min.as_f64() * width),
        y_min: Millimeters::new(y + field.region.y_min.as_f64() * height),
        x_max: Millimeters::new(x + field.region.x_max.as_f64() * width),
        y_max: Millimeters::new(y + field.region.y_max.as_f64() * height),
    }
}

/// Millimeter step shared by the grid and the mm rulers - every cell
/// edge lines up with a ruler tick (and vice versa).
const GRID_STEP_MM: f64 = 50.0;

/// Grid overlay: a faint millimeter lattice across the whole surface,
/// mapped through the calibration like everything else (50 mm cells,
/// matching the ruler's millimeter ticks).
fn draw_grid(
    cr: &cairo::Context,
    cal: &Calibration,
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
) {
    let w_mm = cal.width_mm().as_f64();
    let h_mm = cal.height_mm().as_f64();
    let origin = to_screen(&Vec2::<Millimeters>::new(0.0, 0.0));
    let extent = to_screen(&Vec2::<Millimeters>::new(w_mm, h_mm));

    cr.set_source_rgba(0.8, 0.8, 0.8, 0.12);
    cr.set_line_width(1.0);
    let mut mm = 0.0;
    while mm <= w_mm {
        let from = to_screen(&Vec2::<Millimeters>::new(mm, 0.0));
        let to = to_screen(&Vec2::<Millimeters>::new(mm, h_mm));
        cr.move_to(from.x, from.y);
        cr.line_to(to.x, to.y);
        let _ = cr.stroke();
        mm += GRID_STEP_MM;
    }
    let mut mm = 0.0;
    while mm <= h_mm {
        let from = to_screen(&Vec2::<Millimeters>::new(0.0, mm));
        let to = to_screen(&Vec2::<Millimeters>::new(w_mm, mm));
        cr.move_to(from.x, from.y);
        cr.line_to(to.x, to.y);
        let _ = cr.stroke();
        mm += GRID_STEP_MM;
    }

    // Border of the usable surface, so the lattice visibly bounds the
    // calibrated area instead of dissolving into the canvas edge.
    cr.set_source_rgba(0.8, 0.8, 0.8, 0.25);
    cr.set_line_width(1.5);
    cr.rectangle(
        origin.x.min(extent.x),
        origin.y.min(extent.y),
        (extent.x - origin.x).abs(),
        (extent.y - origin.y).abs(),
    );
    let _ = cr.stroke();
}

/// Ruler overlay: semi-transparent strips on all four edges - pixel
/// ticks on the min edges (left and top), millimeter ticks on the max
/// edges (right and bottom). The mm ticks share [`GRID_STEP_MM`] with
/// the grid overlay, so every grid line lands on a ruler tick.
fn draw_ruler(
    cr: &cairo::Context,
    w: f64,
    h: f64,
    cal: &Calibration,
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
    rotation: DisplayRotation,
) {
    const RULER_W: f64 = 56.0;
    const RULER_H: f64 = 36.0;

    cr.set_source_rgba(0.04, 0.04, 0.06, 0.5);
    cr.rectangle(0.0, 0.0, RULER_W, h);
    cr.rectangle(w - RULER_W, 0.0, RULER_W, h);
    cr.rectangle(0.0, 0.0, w, RULER_H);
    cr.rectangle(0.0, h - RULER_H, w, RULER_H);
    let _ = cr.fill();

    cr.set_font_size(10.0);
    cr.set_source_rgba(0.9, 0.9, 0.9, 0.85);

    // Left edge (x min): pixel ticks every 100 px.
    let mut y = 0.0;
    while y <= h {
        cr.move_to(RULER_W - 8.0, y);
        cr.line_to(RULER_W, y);
        let _ = cr.stroke();
        cr.move_to(4.0, y + 3.0);
        let _ = cr.show_text(&format!("{y} px"));
        y += 100.0;
    }

    // Top edge (y min): pixel ticks every 100 px.
    let mut x = 0.0;
    while x <= w {
        cr.move_to(x, RULER_H - 8.0);
        cr.line_to(x, RULER_H);
        let _ = cr.stroke();
        cr.move_to(x + 3.0, 12.0);
        let _ = cr.show_text(&format!("{x} px"));
        x += 100.0;
    }

    // The mm tick axes follow the display rotation: at 90°/270° the
    // surface axes swap, so screen X walks the native y axis and
    // screen Y walks the native x axis (`Deg270` maps x_mm onto
    // screen Y - walking x_mm for the bottom edge collapsed every
    // tick onto the right edge).
    let extent_x = if rotation.swaps_axes() {
        cal.height_mm().as_f64()
    } else {
        cal.width_mm().as_f64()
    };
    let extent_y = if rotation.swaps_axes() {
        cal.width_mm().as_f64()
    } else {
        cal.height_mm().as_f64()
    };

    // Bottom edge (y max): millimeter ticks along screen X.
    let mut mm = 0.0;
    while mm <= extent_x {
        let probe = if rotation.swaps_axes() {
            Vec2::<Millimeters>::new(0.0, mm)
        } else {
            Vec2::<Millimeters>::new(mm, 0.0)
        };
        let x = to_screen(&probe).x;
        if (0.0..=w).contains(&x) {
            cr.move_to(x, h - 8.0);
            cr.line_to(x, h);
            let _ = cr.stroke();
            cr.move_to(x + 3.0, h - 4.0);
            let _ = cr.show_text(&format!("{mm} mm"));
        }
        mm += GRID_STEP_MM;
    }

    // Right edge (x max): millimeter ticks along screen Y, labels
    // rotated to read bottom-to-top inside the strip.
    let mut mm = 0.0;
    while mm <= extent_y {
        let probe = if rotation.swaps_axes() {
            Vec2::<Millimeters>::new(mm, 0.0)
        } else {
            Vec2::<Millimeters>::new(0.0, mm)
        };
        let y = to_screen(&probe).y;
        if (0.0..=h).contains(&y) {
            cr.move_to(w - 8.0, y);
            cr.line_to(w, y);
            let _ = cr.stroke();
            let _ = cr.save();
            cr.translate(w - 12.0, y + 4.0);
            cr.rotate(-std::f64::consts::FRAC_PI_2);
            let _ = cr.show_text(&format!("{mm} mm"));
            let _ = cr.restore();
        }
        mm += GRID_STEP_MM;
    }
}

/// Positions overlay: mm + pixel coordinates at every touch point,
/// tangible center and field-set field.
fn draw_positions(
    cr: &cairo::Context,
    touches: TouchSet<'_>,
    tracked: TrackedSet<'_>,
    field_sets: &[FieldSet],
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
) {
    cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(11.0);
    cr.set_source_rgba(0.85, 0.85, 0.85, 0.85);

    for touch in touches.iter() {
        draw_position_label(cr, &touch.pos, to_screen, 10.0, -8.0);
    }
    for tangible in tracked.iter() {
        draw_position_label(cr, &tangible.center, to_screen, 10.0, 18.0);
    }
    for set in field_sets {
        for field in &set.fields {
            let region = field_screen_region(set, field);
            draw_position_label(cr, &region.center(), to_screen, 10.0, -8.0);
        }
    }
}

/// Writes the `x_mm,y_mm mm | x_px,y_px px` label at `pos` offset by
/// `dx`/`dy` screen pixels.
fn draw_position_label(
    cr: &cairo::Context,
    pos: &Vec2<Millimeters>,
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
    dx: f64,
    dy: f64,
) {
    let px = to_screen(pos);
    cr.move_to(px.x + dx, px.y + dy);
    let _ = cr.show_text(&fl!(
        language_loader(),
        "position-label",
        x_mm = (pos.x as i64),
        y_mm = (pos.y as i64),
        x_px = (px.x as i64),
        y_px = (px.y as i64)
    ));
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
        let _ = cr.show_text(&fl!(language_loader(), "place-figure"));
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
        fl!(language_loader(), "wait-for-points", count = (count as i64))
    } else if s.stable {
        fl!(language_loader(), "stable-status", count = (count as i64))
    } else {
        fl!(
            language_loader(),
            "stabilizing-status",
            stable = (s.stable_frames as i64),
            threshold = (STABLE_THRESHOLD as i64),
            count = (count as i64)
        )
    };
    let _ = cr.show_text(&status_text);

    // Coordinates next to the points.
    cr.set_font_size(12.0);
    cr.set_source_rgb(0.7, 0.7, 0.7);
    for (i, tp) in touches.iter().enumerate() {
        let p = to_screen(&tp.pos);
        cr.move_to(p.x + 16.0, p.y - 12.0);
        let _ = cr.show_text(&fl!(
            language_loader(),
            "pin-label",
            index = (i as i64),
            x = (tp.pos.x.round() as i64),
            y = (tp.pos.y.round() as i64)
        ));
    }
}

/// Live view: matched figures with names + unmatched points dimmed.
fn draw_live(
    cr: &cairo::Context,
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
}
