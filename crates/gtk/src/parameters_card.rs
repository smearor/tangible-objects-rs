//! The parameters card - live detection tuning (concepts/planned/
//! GUI.md §5): completion mode/max-missing/radius, filter stability,
//! legacy event policy, edge-margin filter. Each row is a linked
//! `ToggleButton` group; clicks send `ControlCommand`s over the same
//! channel the interfaces use, the sync tick reflects external
//! changes back into the buttons.

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use gtk4::Box as GtkBox;
use gtk4::Grid;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::Separator;
use gtk4::ToggleButton;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use tangible_objects_engine::LegacyMode;
use tangible_objects_model::CompletionMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Millimeters;
use tangible_objects_runtime::ControlCommand;

use crate::language_loader;
use crate::shared_state::SharedState;
use crate::shared_state::send_command_built;

/// A linked button row: the group plus the last synced value.
struct Row<T: Copy + PartialEq> {
    /// Row buttons in `values` order.
    buttons: Vec<(ToggleButton, T)>,
    /// The last value the sync tick applied.
    applied: Option<T>,
}

impl<T: Copy + PartialEq> Row<T> {
    /// Activates the button for `value` when it changed since the last
    /// sync (`set_active` re-fires `toggled` - the re-sent command is
    /// idempotent).
    fn sync(&mut self, value: T) {
        if self.applied == Some(value) {
            return;
        }
        self.applied = Some(value);
        for (button, candidate) in &self.buttons {
            button.set_active(*candidate == value);
        }
    }
}

/// Builds the parameters-card content: six `label | buttons` rows and
/// the GUI-local profile-bounds debugging toggle (GUI.md §5's
/// "profile bounding boxes on/off").
pub fn create_parameters_card(state: &Arc<Mutex<SharedState>>) -> GtkBox {
    let grid = Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(6);

    // Completion rows merge the untouched fields out of the live
    // config - `SetCompletion` replaces the whole struct.
    let mut completion_mode = add_row(
        &grid,
        0,
        state,
        &fl!(language_loader(), "param-completion-mode"),
        &[
            (CompletionMode::Match, fl!(language_loader(), "param-match")),
            (CompletionMode::Exact, fl!(language_loader(), "param-exact")),
        ],
        |state, mode| {
            let mut config = state.completion;
            config.mode = mode;
            ControlCommand::SetCompletion(config)
        },
    );
    let mut completion_missing = add_row(
        &grid,
        1,
        state,
        &fl!(language_loader(), "param-max-missing"),
        &[
            (0u32, "0".to_string()),
            (1u32, "1".to_string()),
            (2u32, "2".to_string()),
        ],
        |state, missing| {
            let mut config = state.completion;
            config.max_missing = missing;
            ControlCommand::SetCompletion(config)
        },
    );
    let mut completion_radius = add_row(
        &grid,
        2,
        state,
        &fl!(language_loader(), "param-radius"),
        &[
            (1.0f64, "1.0".to_string()),
            (1.5f64, "1.5".to_string()),
            (2.0f64, "2.0".to_string()),
            (2.5f64, "2.5".to_string()),
            (3.0f64, "3.0".to_string()),
        ],
        |state, radius| {
            let mut config = state.completion;
            config.radius_factor = radius;
            ControlCommand::SetCompletion(config)
        },
    );

    // Filter pipeline parameters - honored under `--filter`, display
    // state in the GUI (same as the filter-mode buttons).
    let mut stability = add_row(
        &grid,
        3,
        state,
        &fl!(language_loader(), "param-stability"),
        &(0u64..=5)
            .map(|frames| (frames, frames.to_string()))
            .collect::<Vec<_>>(),
        |state, frames| ControlCommand::SetFilterMode {
            mode: state.filter_mode,
            stability: Some(FrameCount::new(frames)),
            stage: None,
        },
    );
    let mut legacy = add_row(
        &grid,
        4,
        state,
        &fl!(language_loader(), "param-legacy"),
        &[
            (LegacyMode::Auto, "auto".to_string()),
            (LegacyMode::Always, "always".to_string()),
            (LegacyMode::Never, "never".to_string()),
        ],
        |_, mode| ControlCommand::SetLegacyMode(mode),
    );
    let mut edge = add_row(
        &grid,
        5,
        state,
        &fl!(language_loader(), "param-edge"),
        &[0.0, 5.0, 10.0, 20.0, 40.0]
            .into_iter()
            .map(|mm| (Millimeters::new(mm), format!("{mm}")))
            .collect::<Vec<_>>(),
        |_, margin| ControlCommand::SetEdgeMargin(margin),
    );

    // GUI-local display toggle (GUI.md §5: no control command needed).
    let bounds_toggle = ToggleButton::with_label(&fl!(language_loader(), "toggle-profile-bounds"));
    let bounds_toggle_sync = bounds_toggle.clone();
    bounds_toggle.set_active(
        state
            .lock()
            .map(|s| s.profile_bounds_visible)
            .unwrap_or_default(),
    );
    let state_bounds = state.clone();
    bounds_toggle.connect_toggled(move |button| {
        if let Ok(mut s) = state_bounds.try_lock() {
            s.profile_bounds_visible = button.is_active();
        }
    });

    let state_sync = state.clone();
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        // Read under lock, drop the guard BEFORE `Row::sync` -
        // `set_active` re-fires `connect_toggled`, whose command
        // build locks `state` again (non-recursive mutex
        // self-deadlock, seen in production).
        let Ok(s) = state_sync.lock() else {
            return ControlFlow::Continue;
        };
        let mode = s.completion.mode;
        let missing = s.completion.max_missing;
        let radius = s.completion.radius_factor;
        let stability_frames = s.filter_stability;
        let legacy_mode = s.legacy_mode;
        let margin = s.edge_margin;
        let bounds = s.profile_bounds_visible;
        drop(s);
        completion_mode.sync(mode);
        completion_missing.sync(missing);
        completion_radius.sync(radius);
        stability.sync(stability_frames);
        legacy.sync(legacy_mode);
        edge.sync(margin);
        if bounds != bounds_toggle_sync.is_active() {
            bounds_toggle_sync.set_active(bounds);
        }
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 6);
    body.append(&grid);
    body.append(&Separator::new(Orientation::Horizontal));
    body.append(&bounds_toggle);
    body
}

/// Builds a `name | button-group` row: one checked at a time, clicks
/// send the `ControlCommand` the closure builds.
fn add_row<T: Copy + PartialEq + 'static>(
    grid: &Grid,
    row: i32,
    state: &Arc<Mutex<SharedState>>,
    name: &str,
    values: &[(T, String)],
    command: impl Fn(&SharedState, T) -> ControlCommand + 'static,
) -> Row<T> {
    let command = std::rc::Rc::new(command);
    let name_label = Label::new(Some(name));
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_opacity(0.6);
    grid.attach(&name_label, 0, row, 1, 1);

    let button_box = GtkBox::new(Orientation::Horizontal, 0);
    button_box.add_css_class("linked");
    grid.attach(&button_box, 1, row, 1, 1);

    let mut group: Option<ToggleButton> = None;
    let mut buttons: Vec<(ToggleButton, T)> = Vec::new();
    for &(value, ref label) in values {
        let button = ToggleButton::with_label(label);
        if let Some(leader) = &group {
            button.set_group(Some(leader));
        } else {
            group = Some(button.clone());
        }
        let state_click = state.clone();
        let command = command.clone();
        button.connect_toggled(move |b| {
            if b.is_active() {
                send_command_built(&state_click, |s| command(s, value));
            }
        });
        button_box.append(&button);
        buttons.push((button, value));
    }

    Row {
        buttons,
        applied: None,
    }
}
