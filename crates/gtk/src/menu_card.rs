//! The main-menu card - the always-open root card whose toggle list
//! opens and closes the other cards on the board (GUI.md). Its Arrange
//! action tidies the visible cards via [`crate::card_layout::arrange_strategy`];
//! folded, the card collapses to a round burger-menu FAB.

use std::cell::RefCell;
use std::rc::Rc;

use floating_board::CardId;
use floating_board::FloatingBoard;
use gtk4::Box as GtkBox;
use gtk4::Button;
use gtk4::Image;
use gtk4::Orientation;
use gtk4::ToggleButton;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use tracing::warn;

use crate::card_layout::arrange_strategy;
use crate::language_loader;

/// The menu card's own id - it is skipped in its toggle list so it can
/// never close itself.
pub(crate) const MENU_CARD_ID: &str = "menu";

/// Builds the menu-card content (toggle list plus Arrange action) and
/// its folded-content FAB. The list rebuilds on
/// `card-added`/`card-removed` (the training card joins and leaves
/// with the mode), toggle states re-sync on `card-visibility-changed`
/// (a card closed via its X button unchecks its menu entry). The FAB
/// is display-only - the card's `unfold_on_click` unfolds it on tap
/// and `WholeCardFolded` drags it (a real `Button` would claim the
/// press and break both).
pub(crate) fn create_menu_card(board: &FloatingBoard) -> (GtkBox, GtkBox) {
    let outer = GtkBox::new(Orientation::Vertical, 6);
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

    // Arrange: flows the visible cards into tidy rows (animated) -
    // `None` selects all visible, non-sticky cards; a `DoesNotFit`
    // report leaves the board untouched.
    let arrange = Button::with_label(&fl!(language_loader(), "menu-arrange"));
    let board_arrange = board.clone();
    arrange.connect_clicked(move |_| {
        if let Err(error) = board_arrange.apply_layout(&arrange_strategy(), None, true) {
            warn!(%error, "card arrangement does not fit");
        }
    });

    outer.append(&body);
    outer.append(&arrange);

    // The folded content: a round burger-menu FAB. Display-only, so
    // no child gesture competes with the card's unfold tap and drag.
    let fab = GtkBox::new(Orientation::Vertical, 0);
    fab.add_css_class("round-button");
    fab.set_tooltip_text(Some(&fl!(language_loader(), "card-menu-title")));
    let icon = Image::from_icon_name("open-menu-symbolic");
    icon.set_hexpand(true);
    icon.set_vexpand(true);
    fab.append(&icon);

    (outer, fab)
}
