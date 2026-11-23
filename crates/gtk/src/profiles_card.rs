//! The profiles card - the loaded figure profiles (name, owner,
//! enabled state, current surface position) as a movable card on the
//! `FloatingBoard` (concepts/planned/GUI.md §3).

use std::cell::Cell;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use gtk4::Align;
use gtk4::Box as GtkBox;
use gtk4::Button;
use gtk4::ColorDialog;
use gtk4::ColorDialogButton;
use gtk4::Grid;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::Switch;
use gtk4::glib::ControlFlow;
use gtk4::glib::SignalHandlerId;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use smearor_commons_color::Color;
use tangible_objects_model::FigureColor;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;
use tangible_objects_runtime::ControlCommand;

use crate::folded_badge::count_badge;
use crate::language_loader;
use crate::shared_state::SharedState;
use crate::shared_state::send_command;

/// Widgets of one profile row - kept across ticks so only the
/// volatile cells rewrite.
struct ProfileRow {
    /// The profile this row displays.
    id: PatternId,
    /// Editable name cell - the Save button commits changed texts
    /// as renames.
    name: gtk4::Entry,
    /// Pin count cell - the figure's contact count.
    points: Label,
    /// Registering application cell.
    owner: Label,
    /// Enable switch - sends `SetProfileEnabled`.
    enabled: Switch,
    /// `state-set` connection - the sync blocks it while writing
    /// the switch so `set_active` cannot echo `SetProfileEnabled`
    /// back to the device loop.
    enabled_signal: SignalHandlerId,
    /// Toggle the user requested (`SetProfileEnabled` was sent) but
    /// `s.profiles` has not confirmed yet - the sync leaves the
    /// switch alone while it pends so it cannot snap back.
    enabled_pending: Rc<Cell<Option<(bool, Instant)>>>,
    /// Color preview + picker - sends `SetProfileColor`.
    color: gtk4::ColorDialogButton,
    /// Current surface position cell.
    position: Label,
}

/// Builds the profiles-card content: a `Name | Owner | Enabled |
/// Position` grid refreshed from [`SharedState`] every 100 ms. Rows
/// rebuild only when the registered set changes; the volatile cells
/// (enabled flag, live position) update in place. The folded-content
/// badge shows `enabled/registered` profiles alongside the card body.
pub fn create_profiles_card(
    state: &Arc<Mutex<SharedState>>,
    profiles_dir: Option<PathBuf>,
) -> (GtkBox, GtkBox) {
    let grid = Grid::new();
    grid.set_column_spacing(10);
    grid.set_row_spacing(4);
    for (column, header) in [
        fl!(language_loader(), "col-name"),
        fl!(language_loader(), "col-points"),
        fl!(language_loader(), "col-owner"),
        fl!(language_loader(), "col-enabled"),
        fl!(language_loader(), "col-color"),
        fl!(language_loader(), "col-position"),
    ]
    .iter()
    .enumerate()
    {
        let label = Label::new(Some(header));
        label.set_halign(Align::Start);
        label.add_css_class("dim-label");
        grid.attach(&label, column as i32, 0, 1, 1);
    }

    let (badge, badge_count) = count_badge(
        "applications-games-symbolic",
        &fl!(language_loader(), "card-profiles-title"),
    );

    let rows: Rc<RefCell<Vec<ProfileRow>>> = Rc::new(RefCell::new(Vec::new()));
    let state_tick = state.clone();
    let rows_tick = rows.clone();
    let grid_tick = grid.clone();
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let grid = &grid_tick;
        let s = match state_tick.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        let enabled = s.profiles.iter().filter(|profile| profile.enabled).count();
        badge_count.set_label(&format!("{enabled} / {}", s.profiles.len()));
        let mut rows = rows_tick.borrow_mut();
        if rows
            .iter()
            .map(|row| row.id)
            .ne(s.profiles.iter().map(|profile| profile.pattern_id))
        {
            // Registration set changed - drop every data row below the
            // header and rebuild in `s.profiles` order.
            let row_count = rows.len() as i32;
            rows.clear();
            for _ in 0..row_count {
                grid.remove_row(1);
            }
            for (index, profile) in s.profiles.iter().enumerate() {
                let name = gtk4::Entry::new();
                name.set_text(profile.name.as_str());
                name.set_width_chars(14);
                let points = Label::new(None);
                points.set_halign(Align::Start);
                let owner = Label::new(None);
                owner.set_halign(Align::Start);
                let enabled = Switch::new();
                enabled.set_halign(Align::Start);
                let pattern_id = profile.pattern_id;
                let state_enabled = state_tick.clone();
                let enabled_pending = Rc::new(Cell::new(None));
                let enabled_signal = {
                    let enabled_pending = enabled_pending.clone();
                    enabled.connect_state_set(move |_, active| {
                        enabled_pending.set(Some((active, Instant::now())));
                        send_command(
                            &state_enabled,
                            ControlCommand::SetProfileEnabled {
                                id: pattern_id,
                                enabled: active,
                            },
                        );
                        gtk4::glib::Propagation::Proceed
                    })
                };
                let color = ColorDialogButton::new(Some(ColorDialog::new()));
                let accent = crate::draw::figure_color(profile.pattern_id);
                color.set_rgba(&accent.into());
                color.set_halign(Align::Start);
                let pattern_id_color = profile.pattern_id;
                let state_color = state_tick.clone();
                color.connect_notify_local(Some("rgba"), move |button, _| {
                    send_command(
                        &state_color,
                        ControlCommand::SetProfileColor {
                            id: pattern_id_color,
                            color: FigureColor::from(Color::<f64>::from(button.rgba())),
                        },
                    );
                });
                let position = Label::new(None);
                position.set_halign(Align::Start);
                let row = index as i32 + 1;
                grid.attach(&name, 0, row, 1, 1);
                grid.attach(&points, 1, row, 1, 1);
                grid.attach(&owner, 2, row, 1, 1);
                grid.attach(&enabled, 3, row, 1, 1);
                grid.attach(&color, 4, row, 1, 1);
                grid.attach(&position, 5, row, 1, 1);
                rows.push(ProfileRow {
                    id: profile.pattern_id,
                    name,
                    points,
                    owner,
                    enabled,
                    enabled_signal,
                    enabled_pending,
                    color,
                    position,
                });
            }
        }
        for (profile, row) in s.profiles.iter().zip(rows.iter()) {
            // Sync the entry only while the user is not editing it -
            // a rewrite under an active cursor would drop keystrokes.
            if row.name.text() != profile.name.as_str() && !row.name.has_focus() {
                row.name.set_text(profile.name.as_str());
            }
            row.points.set_label(&profile.num_points.to_string());
            // Reflect the live color (declared `color=` or a
            // `SetProfileColor` from elsewhere).
            let expected = s
                .pattern_styles
                .iter()
                .find(|style| style.pattern_id == profile.pattern_id)
                .and_then(|style| style.color)
                .map(|color| color.0)
                .unwrap_or_else(|| crate::draw::figure_color(profile.pattern_id));
            let current = row.color.rgba();
            let epsilon = 1.0 / 255.0;
            if (current.red() as f64 - expected.r).abs() > epsilon
                || (current.green() as f64 - expected.g).abs() > epsilon
                || (current.blue() as f64 - expected.b).abs() > epsilon
            {
                row.color.set_rgba(&expected.into());
            }
            row.owner.set_label(
                profile
                    .owner
                    .as_ref()
                    .map(|owner| owner.as_str())
                    .unwrap_or("–"),
            );
            // `set_active` emits `state-set` too and cancels an
            // in-flight toggle animation even on an unchanged value
            // (`gtk_switch_end_toggle_animation` runs before the
            // value check) - a write inside the user's ~100 ms toggle
            // window would eat the `state-set` before the command
            // ever sends. Skip writes while a user toggle is
            // unconfirmed and block the handler around real syncs.
            let confirmed = match row.enabled_pending.get() {
                Some((requested, _)) if requested == profile.enabled => true,
                Some((_, at)) if at.elapsed() < Duration::from_millis(500) => false,
                _ => true,
            };
            if confirmed {
                row.enabled_pending.set(None);
                if row.enabled.is_active() != profile.enabled {
                    row.enabled.block_signal(&row.enabled_signal);
                    row.enabled.set_active(profile.enabled);
                    row.enabled.unblock_signal(&row.enabled_signal);
                }
            }
            let position = s
                .tracked
                .iter()
                .find(|tangible| tangible.pattern_id == profile.pattern_id)
                .map(|tangible| format!("{:.0},{:.0} mm", tangible.center.x, tangible.center.y))
                .unwrap_or_else(|| "–".to_string());
            row.position.set_label(&position);
        }
        ControlFlow::Continue
    });

    // Persists the runtime `enabled`/`color` overrides of every row
    // back into the profile files (`SaveProfiles` drains through the
    // device loop like the switches' commands do). Name entries that
    // differ from the live name travel as renames - files move to
    // `<new-name>.toml` and the pattern keeps its `PatternId`.
    let save = Button::with_label(&fl!(language_loader(), "profiles-save"));
    save.set_halign(Align::End);
    let state_save = state.clone();
    let rows_save = rows.clone();
    save.connect_clicked(move |_| {
        let renames = {
            let Ok(s) = state_save.lock() else {
                return;
            };
            rows_save
                .borrow()
                .iter()
                .filter_map(|row| {
                    s.profiles
                        .iter()
                        .find(|profile| profile.pattern_id == row.id)
                        .filter(|profile| {
                            let text = row.name.text();
                            !text.is_empty() && text != profile.name.as_str()
                        })
                        .map(|profile| {
                            (
                                profile.pattern_id,
                                PatternName::new(row.name.text().as_str()),
                            )
                        })
                })
                .collect()
        };
        send_command(
            &state_save,
            ControlCommand::SaveProfiles {
                profiles_dir: profiles_dir.clone(),
                renames,
            },
        );
    });

    let body = GtkBox::new(Orientation::Vertical, 6);
    body.append(&grid);
    body.append(&save);
    (body, badge)
}
