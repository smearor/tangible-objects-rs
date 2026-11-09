//! Card layout persistence - `gui-cards.toml` under
//! `$XDG_CONFIG_HOME/tangible-objects/` (GUI.md §3 Persistence):
//! position, rotation and scale of every `FloatingBoard` card
//! restore on the next GUI start.

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use floating_board::BoardLayout;
use floating_board::Card;
use floating_board::CardPosition;
use floating_board::FloatingBoard;
use gtk4::Orientation;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use tracing::warn;

/// Layout file name inside the `tangible-objects` config dir.
const LAYOUT_FILE: &str = "gui-cards.toml";
/// Minimum distance of the default arrangement from every board edge.
const EDGE_MARGIN: f64 = 100.0;
/// Gap between neighboring cards in the default row arrangement.
const CARD_GAP: f64 = 24.0;
/// Debounce window for writing `layout_snapshot()` back - card
/// gestures emit `geometry-changed` continuously while dragging.
const SAVE_DEBOUNCE: Duration = Duration::from_millis(500);

/// The layout path: `$XDG_CONFIG_HOME/tangible-objects/gui-cards.toml`,
/// falling back to `~/.config/tangible-objects/gui-cards.toml` (same
/// resolution the daemon's runtime store uses).
fn layout_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("tangible-objects").join(LAYOUT_FILE))
}

/// Loads the persisted layout; `None` when the file is missing,
/// unreadable or holds no placements.
fn load_layout() -> Option<BoardLayout> {
    let path = layout_path()?;
    let content = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&content)
        .map_err(|error| warn!(path = %path.display(), %error, "card layout ignored"))
        .ok()
        .filter(|layout: &BoardLayout| !layout.is_empty())
}

/// Writes the layout snapshot into the config dir (best effort - a
/// read-only home dir only loses persistence).
fn save_layout(layout: &BoardLayout) {
    let Some(path) = layout_path() else {
        return;
    };
    let Some(dir) = path.parent() else {
        return;
    };
    match toml::to_string_pretty(layout) {
        Ok(serialized) => {
            if let Err(error) =
                std::fs::create_dir_all(dir).and_then(|_| std::fs::write(&path, serialized))
            {
                warn!(path = %path.display(), %error, "card layout save failed");
            }
        }
        Err(error) => warn!(%error, "card layout serialization failed"),
    }
}

/// Wires layout persistence for `board`:
/// - restores `gui-cards.toml` once real bounds exist (clamping needs
///   them - earlier restores would collapse every position),
/// - arranges the default side-by-side layout when nothing was saved,
/// - writes `layout_snapshot()` back [`SAVE_DEBOUNCE`] after the last
///   `geometry-changed`.
pub(crate) fn wire_layout(board: &FloatingBoard) {
    let layout = load_layout();
    let board_apply = board.clone();
    // Poll until bounds and card footprints exist - restoring or
    // arranging earlier would clamp every position to the origin.
    gtk4::glib::timeout_add_local(Duration::from_millis(50), move || {
        let allocated = board_apply.width() > 0
            && board_apply.height() > 0
            && board_apply
                .cards()
                .iter()
                .all(|card| card_extent(card).0 > 0.0);
        if !allocated {
            return ControlFlow::Continue;
        }
        match &layout {
            Some(layout) => board_apply.restore_layout(layout),
            None => arrange_default(&board_apply),
        }
        ControlFlow::Break
    });

    let dirty = Rc::new(Cell::new(false));
    let dirty_flag = dirty.clone();
    board.connect_local("geometry-changed", false, move |_| {
        dirty_flag.set(true);
        None
    });
    let board_save = board.clone();
    gtk4::glib::timeout_add_local(SAVE_DEBOUNCE, move || {
        if dirty.replace(false) {
            save_layout(&board_save.layout_snapshot());
        }
        ControlFlow::Continue
    });
}

/// The default layout for first runs: cards side by side starting at
/// (`EDGE_MARGIN`, `EDGE_MARGIN`), wrapping to a new row whenever the
/// next card would leave the board's right margin.
fn arrange_default(board: &FloatingBoard) {
    let board_w = f64::from(board.width());
    let mut x = EDGE_MARGIN;
    let mut y = EDGE_MARGIN;
    let mut row_h = 0.0_f64;
    for card in board.cards() {
        let (w, h) = card_extent(&card);
        if x > EDGE_MARGIN && x + w > board_w - EDGE_MARGIN {
            x = EDGE_MARGIN;
            y += row_h + CARD_GAP;
            row_h = 0.0;
        }
        // `CardGeometry.position` is the card's center point.
        card.set_position(CardPosition::new(x + w / 2.0, y + h / 2.0));
        x += w + CARD_GAP;
        row_h = row_h.max(h);
    }
}

/// A card's occupied extent: the rotated footprint when allocated,
/// else its measured minimum size as a floor estimate.
fn card_extent(card: &Card) -> (f64, f64) {
    let extents = gtk4::cairo::RectangleInt::new(0, 0, 0, 0);
    card.footprint().extents(&extents);
    if extents.width() > 0 && extents.height() > 0 {
        (f64::from(extents.width()), f64::from(extents.height()))
    } else {
        let (w, _, _, _) = card.measure(Orientation::Horizontal, -1);
        let (h, _, _, _) = card.measure(Orientation::Vertical, -1);
        (f64::from(w).max(150.0), f64::from(h).max(80.0))
    }
}
