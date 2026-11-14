//! Touch passthrough - the Wayland `set_input_region` sync that lets
//! touches fall through to the desktop beneath except where a card or
//! a registered dead zone is hit (`SetTouchPassthrough` /
//! `--touch-passthrough`).

use floating_board::FloatingBoard;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Vec2;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tangible_objects_model::Calibration;
use tangible_objects_model::DeadZone;
use tangible_objects_model::DisplayRotation;
use tracing::debug;
use tracing::warn;

use crate::shared_state::SharedState;

/// Wires the touch-passthrough input-region sync.
///
/// While enabled, the surface accepts input on the board's cards and
/// inside registered dead zones; every other touch falls through to
/// the desktop beneath (Wayland `set_input_region`; a no-op
/// elsewhere). Dead zones must stay inside the region so their
/// contacts are swallowed by the overlay - the engine drops them at
/// ingest, so the application below must not receive them either.
/// The region is rebuilt on every tick: card footprints move, rotate
/// and scale with the gestures. Called once from `run_gui` after the
/// `FloatingBoard` exists.
pub fn wire_passthrough(board: &FloatingBoard, state: &Arc<Mutex<SharedState>>) {
    let board_sync = board.clone();
    let state_passthrough = state.clone();
    let mut applied = false;
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        // Read the flag and the zone inputs under lock, drop it before
        // the GTK surface calls - `set_input_region` may dispatch into
        // event handlers that take `state` themselves.
        let (enabled, dead_zones, calibration, rotation) = state_passthrough
            .lock()
            .map(|s| {
                (
                    s.touch_passthrough,
                    s.dead_zones.clone(),
                    s.calibration,
                    s.rotation,
                )
            })
            .unwrap_or_default();
        if enabled {
            // The board must not reset the region on its next
            // allocation - the consumer owns it while passthrough runs.
            board_sync.set_input_region_managed(true);
            if let Some(region) =
                passthrough_region(&board_sync, &dead_zones, calibration, rotation)
                && let Some(surface) = board_sync.native().and_then(|n| n.surface())
            {
                surface.set_input_region(Some(&region));
                applied = true;
            }
            // Debug: dump each card's footprint vs its visible geometry -
            // a card missing from the union is completely input-dead.
            for card in board_sync.cards() {
                let geometry = card.geometry();
                let footprint = card.footprint();
                let extents = gtk4::cairo::RectangleInt::new(0, 0, 0, 0);
                footprint.extents(&extents);
                debug!(
                    card = %card.id(),
                    position = ?geometry.position,
                    rotation = ?geometry.rotation,
                    scale = ?geometry.scale,
                    footprint = ?(extents.x(), extents.y(), extents.width(), extents.height()),
                    visible = %card.is_visible(),
                    "passthrough card footprint"
                );
            }
        } else if applied {
            applied = false;
            board_sync.set_input_region_managed(false);
            if let Some(surface) = board_sync.native().and_then(|n| n.surface()) {
                surface.set_input_region(None);
            }
        }
        ControlFlow::Continue
    });
}

/// The passthrough input region in surface coordinates: the rotated
/// footprints of every card on the board, unioned with the rasterized
/// screen circles of all registered dead zones.
fn passthrough_region(
    board: &FloatingBoard,
    dead_zones: &[DeadZone],
    calibration: Option<Calibration>,
    rotation: DisplayRotation,
) -> Option<gtk4::cairo::Region> {
    let native = board.native()?;
    let region = board.cards_region();
    union_dead_zones(&region, board, dead_zones, calibration, rotation);
    if region.is_empty() {
        return Some(region);
    }
    let origin = board.compute_point(&native, &gtk4::graphene::Point::new(0.0, 0.0))?;
    region.translate(origin.x().round() as i32, origin.y().round() as i32);
    Some(region)
}

/// Unions every dead zone's screen circle into `region` (board
/// coordinates - the card layer spans the same canvas the mm→px
/// mapping draws on, so the existing board→surface translation
/// applies to both).
fn union_dead_zones(
    region: &gtk4::cairo::Region,
    board: &FloatingBoard,
    dead_zones: &[DeadZone],
    calibration: Option<Calibration>,
    rotation: DisplayRotation,
) {
    let Some(calibration) = calibration else {
        return;
    };
    let canvas = Vec2::<Pixels>::new(board.width() as f64, board.height() as f64);
    if canvas.x <= 0.0 || canvas.y <= 0.0 {
        return;
    }
    for zone in dead_zones {
        let center = calibration.mm_to_screen(zone.center, canvas, rotation);
        let radius = dead_zone_radius_px(zone, &calibration, canvas, rotation);
        if radius <= 0.0 {
            continue;
        }
        for strip in circle_rects(center, radius) {
            if let Err(error) = region.union_rectangle(&strip) {
                warn!(%error, zone = %zone.name, "failed to union dead-zone strip");
            }
        }
    }
}

/// The zone's suppression radius in screen pixels - mean of both
/// axes, matching the overlay draw (`draw_dead_zones`) so the input
/// region covers the drawn circle.
fn dead_zone_radius_px(
    zone: &DeadZone,
    calibration: &Calibration,
    canvas: Vec2<Pixels>,
    rotation: DisplayRotation,
) -> f64 {
    let center = calibration.mm_to_screen(zone.center, canvas, rotation);
    let radius = zone.radius.as_f64();
    let rx = calibration
        .mm_to_screen(zone.center + Vec2::new(radius, 0.0), canvas, rotation)
        .distance(&center)
        .as_f64();
    let ry = calibration
        .mm_to_screen(zone.center + Vec2::new(0.0, radius), canvas, rotation)
        .distance(&center)
        .as_f64();
    (rx + ry) / 2.0
}

/// Rasterizes a screen circle into 1-px-high [`gtk4::cairo::RectangleInt`]
/// strips - cairo regions are rectangle unions, so the disk becomes one
/// row per scanline. Each row uses the chord at its center-nearest edge
/// (the widest the circle reaches inside that row), so the union covers
/// every point the zone's `contains` would hit.
fn circle_rects(center: Vec2<Pixels>, radius: f64) -> Vec<gtk4::cairo::RectangleInt> {
    let top = (center.y - radius).floor() as i32;
    let bottom = (center.y + radius).ceil() as i32;
    (top..bottom)
        .filter_map(|y| {
            // Distance from the center to the row's nearest edge - zero
            // when the row straddles the center line.
            let nearest = (f64::from(y) - center.y)
                .max(center.y - f64::from(y + 1))
                .max(0.0);
            if nearest > radius {
                return None;
            }
            let half = (radius * radius - nearest * nearest).sqrt().max(0.5);
            let left = (center.x - half).floor() as i32;
            let right = (center.x + half).ceil() as i32;
            Some(gtk4::cairo::RectangleInt::new(left, y, right - left, 1))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::circle_rects;
    use smearor_commons_geometry::Pixels;
    use smearor_commons_geometry::Vec2;

    /// Every disk point lies inside a generated strip; each strip is a
    /// 1-px row inside the circle's bounding box.
    #[test]
    fn circle_rects_cover_the_disk() {
        let center = Vec2::<Pixels>::new(100.4, 80.6);
        let radius = 30.0;
        let strips = circle_rects(center, radius);
        assert!(!strips.is_empty());
        assert!(strips.iter().all(|strip| strip.height() == 1));
        assert!(strips.iter().all(|strip| {
            strip.y() >= (center.y - radius).floor() as i32
                && strip.y() < (center.y + radius).ceil() as i32
        }));

        let mut y = center.y - radius;
        while y <= center.y + radius {
            let mut x = center.x - radius;
            while x <= center.x + radius {
                if (x - center.x).powi(2) + (y - center.y).powi(2) <= radius * radius {
                    let covered = strips.iter().any(|strip| {
                        x >= f64::from(strip.x())
                            && x < f64::from(strip.x() + strip.width())
                            && y >= f64::from(strip.y())
                            && y < f64::from(strip.y() + 1)
                    });
                    assert!(covered, "disk point ({x}, {y}) not covered");
                }
                x += 0.25;
            }
            y += 0.25;
        }
    }

    /// A degenerate radius produces no strips instead of a phantom row.
    #[test]
    fn circle_rects_empty_for_zero_radius() {
        assert!(circle_rects(Vec2::<Pixels>::new(10.0, 10.0), 0.0).is_empty());
    }
}
