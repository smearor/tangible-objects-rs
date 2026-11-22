//! The spell-training card - gesture capture (name entry, arming
//! selection, start button, repetition progress) as a movable card
//! on the `FloatingBoard` (SPELLS.md's learn-by-repetition).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use floating_board::BoundsMode;
use floating_board::CardAnchor;
use floating_board::CardColor;
use floating_board::CardConfig;
use floating_board::CardId;
use floating_board::CardPalette;
use floating_board::CardPlacement;
use floating_board::FloatingBoard;
use smearor_commons_color::Color;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Pixels;

use gtk4::Align;
use gtk4::Box as GtkBox;
use gtk4::Button;
use gtk4::DropDown;
use gtk4::Entry;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::StringList;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use tangible_objects_engine::SPELL_TRAINING_REPETITIONS;
use tangible_objects_model::HandName;
use tangible_objects_model::PatternName;
use tangible_objects_model::Region;
use tangible_objects_model::SpellArming;
use tangible_objects_model::SpellName;
use tangible_objects_runtime::ControlCommand;

use crate::gui_mode::GuiMode;
use crate::language_loader;
use crate::shared_state::SharedState;
use crate::shared_state::send_command;

/// Arming kinds the card offers - the parameter entry carries the
/// region or pattern the selected kind needs.
const ARMING_OPTIONS: [&str; 5] = [
    "free",
    "exclusive-zone",
    "start-spot",
    "five-finger-tap",
    "tangible-chord",
];

/// Parses the parameter entry into the selected arming: `x_min y_min
/// x_max y_max` (mm, comma or space separated) for the region-based
/// kinds, a pattern name for `tangible-chord` (empty = any
/// tangible), ignored for `free`/`five-finger-tap`.
fn parse_arming(selected: u32, param: &str) -> Result<SpellArming, String> {
    let kind = ARMING_OPTIONS
        .get(selected as usize)
        .copied()
        .unwrap_or("free");
    let param = param.trim();
    match kind {
        "free" => Ok(SpellArming::Free),
        "five-finger-tap" => Ok(SpellArming::FiveFingerTap {
            hand: (!param.is_empty()).then(|| HandName::new(param)),
        }),
        "tangible-chord" => Ok(SpellArming::TangibleChord {
            pattern: (!param.is_empty()).then(|| PatternName::new(param)),
        }),
        "start-spot" | "exclusive-zone" => {
            let region = parse_region(param).ok_or_else(|| {
                format!("invalid region \"{param}\" - expected x_min,y_min,x_max,y_max")
            })?;
            if kind == "start-spot" {
                Ok(SpellArming::StartSpot { region })
            } else {
                Ok(SpellArming::ExclusiveZone { region })
            }
        }
        _ => Ok(SpellArming::Free),
    }
}

/// Parses four comma/space separated millimeter values into a
/// [`Region`] - the parameter entry's region format.
fn parse_region(param: &str) -> Option<Region> {
    let parts: Vec<f64> = param
        .split([',', ' '])
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<f64>().ok())
        .collect();
    let &[x_min, y_min, x_max, y_max] = parts.as_slice() else {
        return None;
    };
    Some(Region {
        x_min: Millimeters::new(x_min),
        y_min: Millimeters::new(y_min),
        x_max: Millimeters::new(x_max),
        y_max: Millimeters::new(y_max),
    })
}

/// Builds the spell-training card content: spell-name entry, arming
/// dropdown + parameter, a start button sending `TrainSpell` through
/// the command channel, and a status line tracking the captured
/// repetitions.
pub fn create_spell_training_card(state: &Arc<Mutex<SharedState>>) -> GtkBox {
    let name_entry = Entry::new();
    name_entry.set_placeholder_text(Some(&fl!(language_loader(), "spell-name-placeholder")));
    name_entry.set_width_chars(20);

    let arming_list = StringList::new(&ARMING_OPTIONS);
    let arming = DropDown::new(Some(arming_list), gtk4::Expression::NONE);
    arming.set_selected(0);

    let param_entry = Entry::new();
    param_entry.set_width_chars(20);
    param_entry.set_sensitive(false);

    // The parameter entry activates for the region/pattern kinds.
    let param_entry_kind = param_entry.clone();
    arming.connect_selected_notify(move |dropdown| {
        let kind = ARMING_OPTIONS
            .get(dropdown.selected() as usize)
            .copied()
            .unwrap_or("free");
        let needs_region = kind == "start-spot" || kind == "exclusive-zone";
        let needs_pattern = kind == "tangible-chord";
        let needs_hand = kind == "five-finger-tap";
        param_entry_kind.set_sensitive(needs_region || needs_pattern || needs_hand);
        let placeholder = if needs_region {
            fl!(language_loader(), "spell-arming-region-placeholder")
        } else if needs_pattern {
            fl!(language_loader(), "spell-arming-pattern-placeholder")
        } else if needs_hand {
            fl!(language_loader(), "spell-arming-hand-placeholder")
        } else {
            fl!(language_loader(), "spell-arming-none-placeholder")
        };
        param_entry_kind.set_placeholder_text(Some(&placeholder));
    });

    let start_btn = Button::with_label(&fl!(language_loader(), "spell-train-start"));
    let status = Label::new(Some(""));
    status.set_halign(Align::Start);

    let state_start = state.clone();
    let status_start = status.clone();
    let name_entry_start = name_entry.clone();
    let arming_start = arming.clone();
    let param_entry_start = param_entry.clone();
    start_btn.connect_clicked(move |_| {
        let name = name_entry_start.text().to_string();
        if name.trim().is_empty() {
            return;
        }
        match parse_arming(arming_start.selected(), &param_entry_start.text()) {
            Ok(arming) => send_command(
                &state_start,
                ControlCommand::TrainSpell {
                    name: SpellName::new(&name),
                    arming,
                },
            ),
            Err(error) => status_start.set_label(&error),
        }
    });

    let body = GtkBox::new(Orientation::Vertical, 6);
    body.append(&name_entry);
    body.append(&arming);
    body.append(&param_entry);
    body.append(&start_btn);
    body.append(&status);

    // Progress line: repetitions captured while a `TrainSpell`
    // session is armed; `Some -> None` means the session finished
    // and the spell got registered persistently.
    let state_tick = state.clone();
    let mut was_training = false;
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_tick.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        match s.spell_training_progress {
            Some(captured) => {
                was_training = true;
                status.set_label(&fl!(
                    language_loader(),
                    "spell-train-progress",
                    captured = (captured as i64),
                    total = (SPELL_TRAINING_REPETITIONS as i64)
                ));
            }
            None if was_training => {
                was_training = false;
                status.set_label(&fl!(language_loader(), "spell-train-done"));
            }
            None => {
                status.set_label(&fl!(language_loader(), "spell-train-hint"));
            }
        }
        ControlFlow::Continue
    });

    body
}

/// Board id of the spell-training card - the mode watcher
/// adds/removes it like the profile training card.
const SPELL_TRAINING_CARD_ID: &str = "spell-training";

/// The spell-training card's color scheme: a muted violet header
/// marks gesture capture - distinct from the profile card's green.
fn spell_training_palette() -> CardPalette {
    let mut palette = CardPalette::new();
    palette.insert(CardColor::Header, Color::<f64>::new(0.25, 0.15, 0.35));
    palette
}

/// Poll cadence for mode changes (same 100 ms as the other card
/// syncs).
const MODE_POLL: Duration = Duration::from_millis(100);

/// Wires the spell-training card's mode lifecycle: entering
/// spell-training mode adds the card to the board (restoring its
/// last geometry and fold state), leaving removes it again.
pub fn wire_spell_training_card(
    board: &FloatingBoard,
    state: &Arc<Mutex<SharedState>>,
    initial: Option<CardPlacement>,
) {
    let last = Rc::new(RefCell::new(initial));
    let board_tick = board.clone();
    let state_tick = state.clone();
    let card_id = CardId::new(SPELL_TRAINING_CARD_ID);
    let mut mode = state.lock().map(|s| s.mode).unwrap_or(GuiMode::Live);
    if mode == GuiMode::SpellTraining {
        add_spell_training_card(&board_tick, &state_tick, &last);
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
            GuiMode::SpellTraining => {
                add_spell_training_card(&board_tick, &state_tick, &last);
            }
            GuiMode::Live | GuiMode::Training => {
                if let Ok(card) = board_tick.card(&card_id) {
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

/// Adds the spell-training card to `board` and applies the stashed
/// placement (geometry + fold) when one exists.
fn add_spell_training_card(
    board: &FloatingBoard,
    state: &Arc<Mutex<SharedState>>,
    last: &Rc<RefCell<Option<CardPlacement>>>,
) {
    let card_id = CardId::new(SPELL_TRAINING_CARD_ID);
    if board.card(&card_id).is_ok() {
        return;
    }
    let body = create_spell_training_card(state);
    let _ = board.add_card_with_id(
        card_id.clone(),
        &body,
        CardConfig {
            title: Some(fl!(language_loader(), "card-spell-training-title")),
            position: CardAnchor::BottomLeft,
            anchor_margin: Pixels::new(100.0),
            bounds_mode: BoundsMode::Contained,
            content_padding: crate::CARD_CONTENT_PADDING,
            foldable: true,
            resizable: true,
            rotate_grip: true,
            palette: Some(spell_training_palette()),
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
