//! The main-menu card - the always-open root card whose toggle list
//! opens and closes the other cards on the board (GUI.md).

use std::cell::RefCell;
use std::rc::Rc;

use floating_board::CardId;
use floating_board::FloatingBoard;
use gtk4::Box as GtkBox;
use gtk4::Orientation;
use gtk4::ToggleButton;
use gtk4::prelude::*;

/// The menu card's own id - it is skipped in its toggle list so it can
/// never close itself.
pub(crate) const MENU_CARD_ID: &str = "menu";

/// Builds the menu-card content: one toggle per non-menu board card
/// showing that card's header title. The list rebuilds on
/// `card-added`/`card-removed` (the training card joins and leaves
/// with the mode), toggle states re-sync on `card-visibility-changed`
/// (a card closed via its X button unchecks its menu entry).
pub(crate) fn create_menu_card(board: &FloatingBoard) -> GtkBox {
    let body = GtkBox::new(Orientation::Vertical, 4);
    let toggles: Rc<RefCell<Vec<(CardId, ToggleButton)>>> = Rc::new(RefCell::new(Vec::new()));

    let rebuild = {
        let board = board.clone();
        let toggles = toggles.clone();
        let body = body.clone();
        Rc::new(move || {
            while let Some(child) = body.first_child() {
                body.remove(&child);
            }
            let mut toggles = toggles.borrow_mut();
            toggles.clear();
            for card in board.cards() {
                let id = card.id();
                if id == CardId::new(MENU_CARD_ID) {
                    continue;
                }
                let button = ToggleButton::with_label(&card.title());
                button.set_active(board.is_card_visible(&id).unwrap_or(true));
                let board_click = board.clone();
                let id_click = id.clone();
                button.connect_toggled(move |button| {
                    let _ = board_click.set_card_visible(&id_click, button.is_active());
                });
                body.append(&button);
                toggles.push((id, button));
            }
        })
    };

    rebuild();

    // A card added or removed later (training mode) rebuilds the list.
    let rebuild_added = rebuild.clone();
    board.connect_local("card-added", false, move |_| {
        rebuild_added();
        None
    });
    let rebuild_removed = rebuild.clone();
    board.connect_local("card-removed", false, move |_| {
        rebuild_removed();
        None
    });

    // A card hidden or shown outside the menu (its X button) keeps the
    // toggles truthful - the re-fired `connect_toggled` is a no-op
    // because the board already holds the target visibility.
    let toggles_sync = toggles;
    let board_sync = board.clone();
    board.connect_local("card-visibility-changed", false, move |args| {
        if let Ok(card) = args[1].get::<floating_board::Card>() {
            let id = card.id();
            let visible = board_sync.is_card_visible(&id).unwrap_or(true);
            if let Some((_, button)) = toggles_sync
                .borrow()
                .iter()
                .find(|(toggle_id, _)| *toggle_id == id)
                && button.is_active() != visible
            {
                button.set_active(visible);
            }
        }
        None
    });

    body
}
