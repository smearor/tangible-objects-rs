//! The training card - profile capture (name entry, save button,
//! stability status) as a movable card on the `FloatingBoard`
//! (concepts/planned/GUI.md §3).

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use floating_board::BoundsMode;
use floating_board::CardAnchor;
use floating_board::CardConfig;
use floating_board::CardId;
use floating_board::CardPlacement;
use floating_board::FloatingBoard;
use smearor_commons_geometry::Pixels;

use gtk4::Box as GtkBox;
use gtk4::Button;
use gtk4::Entry;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::ProfileFile;
use tracing::error;
use tracing::info;

use crate::evdev_thread::STABLE_THRESHOLD;
use crate::gui_mode::GuiMode;
use crate::language_loader;
use crate::shared_state::SharedState;

/// Tolerance the saved profile takes (scalar mm).
const DEFAULT_TOLERANCE: PatternTolerance = PatternTolerance::new(3.0);

/// Builds the training-card content: name entry + save button + a
/// stability status line. The entry activates in training mode, the
/// save button once the contact set held stable
/// (`STABLE_THRESHOLD` frames, ≥3 points) - the status line explains
/// whichever gate is currently closed.
pub fn create_training_card(
    state: &Arc<Mutex<SharedState>>,
    profiles_dir: &Option<String>,
) -> GtkBox {
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some(&fl!(language_loader(), "figure-name-placeholder")));
    name_entry.set_width_chars(20);
    name_entry.set_sensitive(false);

    let save_btn = Button::with_label(&fl!(language_loader(), "save-profile"));
    save_btn.set_sensitive(false);

    let status = Label::new(Some(""));
    status.set_halign(gtk4::Align::Start);

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
            Ok(()) => info!(path = %filepath.display(), "profile saved"),
            Err(e) => error!(error = %e, "profile save failed"),
        }
    });

    let body = GtkBox::new(Orientation::Vertical, 6);
    body.append(&name_entry);
    body.append(&save_btn);
    body.append(&status);

    // Sensitivity + status text follow the shared state every tick -
    // the mode itself flips via the info card's mode button.
    let state_status = state.clone();
    let name_entry_status = name_entry.clone();
    let save_btn_status = save_btn.clone();
    let mut applied_mode = state_status.lock().map(|s| s.mode).unwrap_or(GuiMode::Live);
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_status.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        if s.mode != applied_mode {
            applied_mode = s.mode;
            name_entry_status.set_sensitive(applied_mode == GuiMode::Training);
        }
        let count = s.touches.len();
        match s.mode {
            GuiMode::Live => {
                status.set_label(&fl!(language_loader(), "train-hint"));
            }
            GuiMode::Training => {
                if let Some(err) = &s.device_error {
                    status.set_label(err);
                } else if count < 3 {
                    status.set_label(&fl!(
                        language_loader(),
                        "wait-for-points",
                        count = (count as i64)
                    ));
                } else if s.stable {
                    status.set_label(&fl!(language_loader(), "stable-hint"));
                } else {
                    status.set_label(&fl!(
                        language_loader(),
                        "stabilizing",
                        stable = (s.stable_frames as i64),
                        threshold = (STABLE_THRESHOLD as i64)
                    ));
                }
            }
        }
        save_btn_status.set_sensitive(s.stable);
        ControlFlow::Continue
    });

    body
}

/// Board id of the training card - the mode watcher adds/removes it,
/// so it stays out of the static card set.
const TRAINING_CARD_ID: &str = "training";

/// Poll cadence for mode changes (same 100 ms as the other card
/// syncs).
const MODE_POLL: Duration = Duration::from_millis(100);

/// Wires the training card's mode lifecycle: entering training mode
/// adds the card to the board (restoring its last geometry and fold
/// state), leaving removes it again. `initial` is the placement the
/// persisted layout held for the card - the first add restores it
/// exactly; later cycles reuse the pose captured at removal.
pub fn wire_training_card(
    board: &FloatingBoard,
    state: &Arc<Mutex<SharedState>>,
    profiles_dir: &Option<String>,
    initial: Option<CardPlacement>,
) {
    let last = Rc::new(RefCell::new(initial));
    let board_tick = board.clone();
    let state_tick = state.clone();
    let profiles_dir_tick = profiles_dir.clone();
    let card_id = CardId::new(TRAINING_CARD_ID);
    let mut mode = state.lock().map(|s| s.mode).unwrap_or(GuiMode::Live);
    if mode == GuiMode::Training {
        add_training_card(&board_tick, &state_tick, &profiles_dir_tick, &last);
    }
    gtk4::glib::timeout_add_local(MODE_POLL, move || {
        let Ok(s) = state_tick.lock() else {
            return ControlFlow::Continue;
        };
        let current = s.mode;
        drop(s);
        if current == mode {
            return ControlFlow::Continue;
        }
        mode = current;
        match mode {
            GuiMode::Training => {
                add_training_card(&board_tick, &state_tick, &profiles_dir_tick, &last);
            }
            GuiMode::Live => {
                if let Ok(card) = board_tick.card(&card_id) {
                    // Stash the pose so re-adding restores it; the
                    // periodic layout save skips absent cards anyway.
                    *last.borrow_mut() = Some(CardPlacement::new(
                        card_id.clone(),
                        card.geometry(),
                        card.folded(),
                    ));
                    let _ = board_tick.remove_card(&card_id);
                }
            }
        }
        ControlFlow::Continue
    });
}

/// Adds the training card to `board` and applies the stashed
/// placement (geometry + fold) when one exists.
fn add_training_card(
    board: &FloatingBoard,
    state: &Arc<Mutex<SharedState>>,
    profiles_dir: &Option<String>,
    last: &Rc<RefCell<Option<CardPlacement>>>,
) {
    let card_id = CardId::new(TRAINING_CARD_ID);
    if board.card(&card_id).is_ok() {
        return;
    }
    let body = create_training_card(state, profiles_dir);
    let _ = board.add_card_with_id(
        card_id.clone(),
        &body,
        CardConfig {
            title: Some(fl!(language_loader(), "card-training-title")),
            position: CardAnchor::BottomLeft,
            anchor_margin: Pixels::new(100.0),
            bounds_mode: BoundsMode::Contained,
            foldable: true,
            ..Default::default()
        },
    );
    let Ok(card) = board.card(&card_id) else {
        return;
    };
    if let Some(placement) = &*last.borrow() {
        card.set_geometry(&placement.geometry);
        card.set_folded(placement.folded);
    }
}
