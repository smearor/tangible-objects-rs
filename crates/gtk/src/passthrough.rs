//! Touch passthrough - the Wayland `set_input_region` sync that lets
//! touches fall through to the desktop beneath except where a card is
//! hit (`SetTouchPassthrough` / `--touch-passthrough`).

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use floating_board::FloatingBoard;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use tracing::debug;

use crate::shared_state::SharedState;

/// Wires the touch-passthrough input-region sync.
///
/// While enabled, the surface accepts input only on the board's
/// cards; every other touch falls through to the desktop beneath
/// (Wayland `set_input_region`; a no-op elsewhere). The region is
/// rebuilt on every tick: card footprints move, rotate and scale with
/// the gestures. Called once from `run_gui` after the `FloatingBoard`
/// exists.
pub fn wire_passthrough(board: &FloatingBoard, state: &Arc<Mutex<SharedState>>) {
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
            if let Some(region) = passthrough_region(&board_sync)
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
/// footprints of every card on the board.
fn passthrough_region(board: &FloatingBoard) -> Option<gtk4::cairo::Region> {
    let native = board.native()?;
    let cards = board.cards_region();
    if cards.is_empty() {
        return Some(cards);
    }
    let origin = board.compute_point(&native, &gtk4::graphene::Point::new(0.0, 0.0))?;
    cards.translate(origin.x().round() as i32, origin.y().round() as i32);
    Some(cards)
}
