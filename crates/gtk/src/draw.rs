//! Cairo drawing - the surface scene per mode (live figures /
//! training capture) plus the frame chrome.

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use i18n_embed_fl::fl;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Vec2;
use tangible_objects_model::Calibration;
use tangible_objects_model::DeadZone;
use tangible_objects_model::DisplayRotation;
use tangible_objects_model::FieldSet;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::PhysicalShape;
use tangible_objects_model::Region;
use tangible_objects_model::SpellMatched;
use tangible_objects_model::TangibleRenderMode;
use tangible_objects_model::TouchSet;
use tangible_objects_model::TrackedSet;

use crate::evdev_thread::STABLE_THRESHOLD;
use crate::gui_mode::GuiMode;
use crate::language_loader;
use crate::pattern_style::PatternStyle;
use crate::shared_state::SharedState;

/// Per-figure accent color - stable hash over the pattern ID.
pub(crate) fn figure_color(id: PatternId) -> (f64, f64, f64) {
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

    if s.spells_visible {
        draw_spells(cr, &s.trajectories, s.last_spell.as_ref(), &to_screen);
    }
    if s.dead_zones_visible {
        draw_dead_zones(cr, &s.dead_zones, touches.into(), &to_screen);
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
        draw_live(
            cr,
            touches.into(),
            matches,
            (&s.tracked).into(),
            &LiveStyle {
                styles: &s.pattern_styles,
                profile_bounds: s.profile_bounds_visible,
                render_mode: s.render_mode,
                canvas: (w, h),
            },
            &to_screen,
        );
    }

    if s.positions_visible {
        draw_positions(
            cr,
            touches.into(),
            matches,
            (&s.tracked).into(),
            &s.field_sets,
            (w, h),
            &to_screen,
        );
    }
    if s.ruler_visible {
        draw_ruler(cr, w, h, &cal, &to_screen, rotation);
    }
}

/// Spell-gesture overlay (SPELLS.md): live finger trajectories as
/// fading polylines plus a flash of the matched spell's name at the
/// gesture centroid - the debug view for the recognition pipeline.
fn draw_spells(
    cr: &cairo::Context,
    trajectories: &[Vec<Vec2<Millimeters>>],
    last_spell: Option<&(SpellMatched, Instant)>,
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
) {
    // Live trajectories: cyan-ish strokes, newest points brightest.
    for (ti, traj) in trajectories.iter().enumerate() {
        let color = figure_color((ti as u32).into());
        cr.set_line_width(3.0);
        cr.set_line_cap(cairo::LineCap::Round);
        for window in traj.windows(2) {
            let a = to_screen(&window[0]);
            let b = to_screen(&window[1]);
            cr.set_source_rgba(color.0, color.1, color.2, 0.7);
            cr.move_to(a.x, a.y);
            cr.line_to(b.x, b.y);
            let _ = cr.stroke();
        }
        // Head marker.
        if let Some(last) = traj.last() {
            let p = to_screen(last);
            cr.set_source_rgba(color.0, color.1, color.2, 0.9);
            cr.arc(p.x, p.y, 6.0, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.fill();
        }
    }

    // Match flash: spell name at the gesture centroid, fading out
    // over ~1.5 s.
    let Some((matched, at)) = last_spell else {
        return;
    };
    let age = at.elapsed().as_secs_f64();
    let alpha = (1.5 - age).clamp(0.0, 1.0);
    if alpha <= 0.0 {
        return;
    }
    let center = to_screen(&matched.center);
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.25 * alpha);
    cr.arc(
        center.x,
        center.y,
        60.0 + age * 40.0,
        0.0,
        2.0 * std::f64::consts::PI,
    );
    let _ = cr.stroke();
    cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    cr.set_font_size(36.0);
    cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
    if let Ok(ext) = cr.text_extents(matched.name.as_str()) {
        cr.move_to(center.x - ext.width() / 2.0, center.y);
        let _ = cr.show_text(matched.name.as_str());
    }
}

/// Dead-zone overlay: a red circle per suppressed spot - unfilled
/// while no touch sits inside, solid-filled while a contact is being
/// suppressed by the zone. Drawn beneath the figure/touch layer.
fn draw_dead_zones(
    cr: &cairo::Context,
    zones: &[DeadZone],
    touches: TouchSet<'_>,
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

        let hit = touches.iter().any(|touch| zone.contains(&touch.pos));
        if hit {
            cr.set_source_rgba(1.0, 0.3, 0.25, 0.45);
            cr.arc(center.x, center.y, radius, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.fill();
        }
        cr.set_source_rgba(1.0, 0.3, 0.25, 0.9);
        cr.set_line_width(2.5);
        cr.arc(center.x, center.y, radius, 0.0, 2.0 * std::f64::consts::PI);
        let _ = cr.stroke();

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

/// Radial label rotation - the text reads upright from the canvas
/// edge nearest the label (station-label pattern: the text-up vector
/// points toward the canvas center... readable by the player on the
/// label's side).
fn radial_angle(x: f64, y: f64, cx: f64, cy: f64) -> f64 {
    (y - cy).atan2(x - cx) - std::f64::consts::FRAC_PI_2
}

/// Draws `text` centered at `(x, y)`, rotated for the nearest canvas
/// edge.
fn draw_radial_text(cr: &cairo::Context, text: &str, x: f64, y: f64, canvas: (f64, f64)) {
    let _ = cr.save();
    cr.translate(x, y);
    cr.rotate(radial_angle(x, y, canvas.0 / 2.0, canvas.1 / 2.0));
    if let Ok(ext) = cr.text_extents(text) {
        cr.move_to(-ext.width() / 2.0, ext.height() / 2.0);
    } else {
        cr.move_to(0.0, 0.0);
    }
    let _ = cr.show_text(text);
    let _ = cr.restore();
}

/// The outward unit direction from `origin` to `pos` - `(0, -1)` when
/// they coincide (label still drifts to the nearest edge).
fn outward_dir(pos: (f64, f64), origin: (f64, f64)) -> (f64, f64) {
    let dx = pos.0 - origin.0;
    let dy = pos.1 - origin.1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-6 {
        (0.0, -1.0)
    } else {
        (dx / len, dy / len)
    }
}

/// A pattern-space point rotated by `rad` and translated to `center`
/// (millimeters) - the tangible pose for physical shapes.
fn pose_point(
    point: &Vec2<Millimeters>,
    center: &Vec2<Millimeters>,
    rad: f64,
) -> Vec2<Millimeters> {
    let (sin, cos) = rad.sin_cos();
    Vec2::<Millimeters>::new(
        center.x + point.x * cos - point.y * sin,
        center.y + point.x * sin + point.y * cos,
    )
}

/// The screen-space radius a physical shape's outline circle takes
/// (mean of the two axes so the circle stays circular under
/// anisotropic calibration).
fn shape_radius_px(
    center: &Vec2<Millimeters>,
    radius: Millimeters,
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
    screen_center: &(f64, f64),
) -> f64 {
    let rx = to_screen(&(*center + Vec2::new(radius.as_f64(), 0.0))).x - screen_center.0;
    let ry = to_screen(&(*center + Vec2::new(0.0, radius.as_f64()))).y - screen_center.1;
    (rx.abs() + ry.abs()) / 2.0
}

/// Traces the declared physical outline of a match (pattern-space
/// shape rotated by the match's `rotation_rad` onto the surface).
fn draw_physical_shape(
    cr: &cairo::Context,
    m: &PatternMatch,
    shape: &PhysicalShape,
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
    color: (f64, f64, f64),
    alpha: f64,
) {
    let (r, g, b) = color;
    let screen_center = to_screen(&m.center);
    let (scx, scy) = (screen_center.x, screen_center.y);
    let rad = m.rotation_rad.as_f64();
    match shape {
        PhysicalShape::Circle { radius } => {
            let r_px = shape_radius_px(&m.center, *radius, to_screen, &(scx, scy));
            cr.arc(scx, scy, r_px, 0.0, 2.0 * std::f64::consts::PI);
        }
        PhysicalShape::Rectangle {
            half_width,
            half_height,
        }
        | PhysicalShape::Ellipse {
            half_width,
            half_height,
        } => {
            let (hw, hh) = (half_width.as_f64(), half_height.as_f64());
            let points: Vec<Vec2<Millimeters>> = match shape {
                PhysicalShape::Rectangle { .. } => vec![
                    Vec2::new(-hw, -hh),
                    Vec2::new(hw, -hh),
                    Vec2::new(hw, hh),
                    Vec2::new(-hw, hh),
                ],
                _ => (0..48)
                    .map(|i| {
                        let t = i as f64 / 48.0 * 2.0 * std::f64::consts::PI;
                        Vec2::new(hw * t.cos(), hh * t.sin())
                    })
                    .collect(),
            };
            for (i, pt) in points.iter().enumerate() {
                let p = to_screen(&pose_point(pt, &m.center, rad));
                if i == 0 {
                    cr.move_to(p.x, p.y);
                } else {
                    cr.line_to(p.x, p.y);
                }
            }
            cr.close_path();
        }
        PhysicalShape::Polygon { points } => {
            for (i, pt) in points.iter().enumerate() {
                let p = to_screen(&pose_point(pt, &m.center, rad));
                if i == 0 {
                    cr.move_to(p.x, p.y);
                } else {
                    cr.line_to(p.x, p.y);
                }
            }
            cr.close_path();
        }
    }
    cr.set_source_rgba(r, g, b, 0.12 * alpha);
    let _ = cr.fill_preserve();
    cr.set_source_rgba(r, g, b, 0.75 * alpha);
    cr.set_line_width(2.0);
    let _ = cr.stroke();
}

/// The declared profile color for `pattern_id`, falling back to the
/// hash palette.
fn match_color(styles: &[PatternStyle], pattern_id: PatternId) -> (f64, f64, f64) {
    styles
        .iter()
        .find(|style| style.pattern_id == pattern_id)
        .and_then(|style| style.color)
        .map(|color| color.rgb())
        .unwrap_or_else(|| figure_color(pattern_id))
}

/// Positions overlay: mm + pixel coordinates at every touch point,
/// tangible center and field-set field - labels sit outside the
/// object along the canvas-radial direction with a leader line,
/// rotated to read from the nearest edge.
fn draw_positions(
    cr: &cairo::Context,
    touches: TouchSet<'_>,
    matches: &[PatternMatch],
    tracked: TrackedSet<'_>,
    field_sets: &[FieldSet],
    canvas: (f64, f64),
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
) {
    cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(11.0);

    // Which match owns each contact - matched labels anchor on the
    // figure center so they clear the object.
    let mut anchor_of: std::collections::HashMap<usize, Vec2<Pixels>> =
        std::collections::HashMap::new();
    let mut extent_of: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
    for m in matches {
        let center = to_screen(&m.center);
        let extent = m
            .touch_indices
            .iter()
            .map(|&idx| {
                let p = to_screen(&touches[idx].pos);
                ((p.x - center.x).powi(2) + (p.y - center.y).powi(2)).sqrt()
            })
            .fold(0.0_f64, f64::max);
        for &idx in &m.touch_indices {
            anchor_of.insert(idx, Vec2::<Pixels>::new(center.x, center.y));
            extent_of.insert(idx, extent);
        }
    }
    let canvas_center = Vec2::<Pixels>::new(canvas.0 / 2.0, canvas.1 / 2.0);

    for (i, touch) in touches.iter().enumerate() {
        let anchor = anchor_of.get(&i).copied().unwrap_or(canvas_center);
        let offset = extent_of.get(&i).copied().unwrap_or(0.0) + 40.0;
        draw_position_label(cr, &touch.pos, &anchor, offset, canvas, to_screen);
    }
    for tangible in tracked.iter() {
        draw_position_label(
            cr,
            &tangible.center,
            &canvas_center,
            40.0,
            canvas,
            to_screen,
        );
    }
    for set in field_sets {
        for field in &set.fields {
            let region = field_screen_region(set, field);
            let anchor = Vec2::<Pixels>::new(region.center().x, region.center().y);
            draw_position_label(cr, &region.center(), &anchor, 40.0, canvas, to_screen);
        }
    }
}

/// Writes the `x_mm,y_mm mm | x_px,y_px px` label at a radial offset
/// from `anchor`, with a leader line back to the point and the text
/// rotated for the nearest canvas edge.
fn draw_position_label(
    cr: &cairo::Context,
    pos: &Vec2<Millimeters>,
    anchor: &Vec2<Pixels>,
    offset: f64,
    canvas: (f64, f64),
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
) {
    let p = to_screen(pos);
    let dir = outward_dir((p.x, p.y), (anchor.x, anchor.y));
    let lx = p.x + dir.0 * offset;
    let ly = p.y + dir.1 * offset;

    cr.set_source_rgba(0.85, 0.85, 0.85, 0.4);
    cr.set_line_width(0.8);
    cr.move_to(p.x, p.y);
    cr.line_to(lx, ly);
    let _ = cr.stroke();

    cr.set_source_rgba(0.85, 0.85, 0.85, 0.85);
    draw_radial_text(
        cr,
        &format!(
            "{},{} mm | {},{} px",
            pos.x.round() as i64,
            pos.y.round() as i64,
            p.x.round() as i64,
            p.y.round() as i64
        ),
        lx,
        ly,
        canvas,
    );
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
/// Render-relevant state bundle for [`draw_live`] - profile styles,
/// debug flags, render mode and canvas dims in one struct so the
/// signature stays small.
struct LiveStyle<'a> {
    /// Declared profile colors/shapes (`SharedState::pattern_styles`).
    styles: &'a [PatternStyle],
    /// Profile bounding-box debug overlay flag.
    profile_bounds: bool,
    /// Which figure layers to draw.
    render_mode: TangibleRenderMode,
    /// Canvas `(width, height)` in pixels.
    canvas: (f64, f64),
}

fn draw_live(
    cr: &cairo::Context,
    touches: TouchSet<'_>,
    matches: &[PatternMatch],
    tracked: TrackedSet<'_>,
    live: &LiveStyle<'_>,
    to_screen: &impl Fn(&Vec2<Millimeters>) -> Vec2<Pixels>,
) {
    let styles = live.styles;
    let profile_bounds = live.profile_bounds;
    let render_mode = live.render_mode;
    let canvas = live.canvas;
    // Collect matched indices.
    let mut matched_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for m in matches {
        for &idx in &m.touch_indices {
            matched_indices.insert(idx);
        }
    }

    // Profile bounding boxes - the Parameters card's debugging
    // toggle draws each matched figure's occupied extent (dashed).
    if profile_bounds {
        for m in matches {
            let mut x_min = f64::INFINITY;
            let mut y_min = f64::INFINITY;
            let mut x_max = f64::NEG_INFINITY;
            let mut y_max = f64::NEG_INFINITY;
            for &idx in &m.touch_indices {
                let p = to_screen(&touches[idx].pos);
                x_min = x_min.min(p.x);
                y_min = y_min.min(p.y);
                x_max = x_max.max(p.x);
                y_max = y_max.max(p.y);
            }
            if !x_min.is_finite() {
                continue;
            }
            let (r, g, b) = figure_color(m.pattern_id);
            cr.set_dash(&[4.0, 4.0], 0.0);
            cr.set_line_width(1.5);
            cr.set_source_rgba(r, g, b, 0.6);
            cr.rectangle(
                x_min - 24.0,
                y_min - 24.0,
                x_max - x_min + 48.0,
                y_max - y_min + 48.0,
            );
            let _ = cr.stroke();
            cr.set_dash(&[], 0.0);
        }
    }

    let draw_points = matches!(
        render_mode,
        TangibleRenderMode::TouchPoints | TangibleRenderMode::TouchPointsAndPhysicalGeometry
    );
    let draw_shape = matches!(
        render_mode,
        TangibleRenderMode::PhysicalGeometry | TangibleRenderMode::TouchPointsAndPhysicalGeometry
    );

    // Draw figures (shape + lines + name).
    for m in matches {
        let (r, g, b) = match_color(styles, m.pattern_id);
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

        // Declared physical outline beneath the contact visuals.
        if draw_shape
            && let Some(shape) = styles
                .iter()
                .find(|style| style.pattern_id == m.pattern_id)
                .and_then(|style| style.shape.as_ref())
        {
            draw_physical_shape(cr, m, shape, to_screen, (r, g, b), alpha);
        }

        if draw_points {
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
        }

        // Figure extent in screen px - the name labels radially
        // outside it, the 0° marker spans half of it.
        let center = to_screen(&m.center);
        let extent = m
            .touch_indices
            .iter()
            .map(|&idx| {
                let p = to_screen(&touches[idx].pos);
                ((p.x - center.x).powi(2) + (p.y - center.y).powi(2)).sqrt()
            })
            .fold(0.0_f64, f64::max);

        // 0° marker: a wedge along the pattern's orientation axis.
        let rad = m.rotation_rad.as_f64();
        let tip_mm =
            Vec2::<Millimeters>::new(m.center.x + rad.cos() * 1.0, m.center.y + rad.sin() * 1.0);
        let tip = to_screen(&tip_mm);
        let dir_len = ((tip.x - center.x).powi(2) + (tip.y - center.y).powi(2)).sqrt();
        if dir_len > 1e-6 {
            let dir = ((tip.x - center.x) / dir_len, (tip.y - center.y) / dir_len);
            let mark = center;
            let mx = mark.x + dir.0 * extent * 0.5;
            let my = mark.y + dir.1 * extent * 0.5;
            cr.set_source_rgba(r, g, b, 0.9 * alpha);
            cr.set_line_width(3.0);
            cr.move_to(mark.x, mark.y);
            cr.line_to(mx, my);
            let _ = cr.stroke();
            cr.arc(mx, my, 4.0, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.fill();
        }

        // Name: radial label outside the figure extent, rotated for
        // the nearest canvas edge.
        let dir = outward_dir((center.x, center.y), (canvas.0 / 2.0, canvas.1 / 2.0));
        let lx = center.x + dir.0 * (extent + 46.0);
        let ly = center.y + dir.1 * (extent + 46.0);
        cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        cr.set_font_size(22.0);
        draw_radial_text(cr, m.pattern_name.as_str(), lx, ly, canvas);
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
