//! The filter card - the `FilterMode` button group as a movable card
//! on the `FloatingBoard` (concepts/planned/GUI.md §3).

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use gtk4::Box as GtkBox;
use gtk4::Orientation;
use gtk4::ToggleButton;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use tangible_objects_model::FilterMode;
use tangible_objects_runtime::ControlCommand;

use crate::language_loader;
use crate::shared_state::SharedState;
use crate::shared_state::send_command;

/// Builds the filter-card content: a linked `FilterMode` button set -
/// one checked at a time, commands flow through the control channel
/// (honored by the proxy in `--filter`; display state otherwise).
pub fn create_filter_card(state: &Arc<Mutex<SharedState>>) -> GtkBox {
    let mode_row = GtkBox::new(Orientation::Vertical, 0);
    mode_row.add_css_class("linked");

    let current_mode = state.lock().map(|s| s.filter_mode).unwrap_or_default();
    let mut mode_group: Option<ToggleButton> = None;
    let mut mode_buttons: Vec<(ToggleButton, FilterMode)> = Vec::new();
    for mode in [
        FilterMode::Off,
        FilterMode::Immediate,
        FilterMode::Gated,
        FilterMode::Guarded,
        FilterMode::TangiblesOnly,
    ] {
        let btn = ToggleButton::with_label(&match mode {
            FilterMode::Off => fl!(language_loader(), "filter-mode-off"),
            FilterMode::Immediate => fl!(language_loader(), "filter-mode-immediate"),
            FilterMode::Gated => fl!(language_loader(), "filter-mode-gated"),
            FilterMode::Guarded => fl!(language_loader(), "filter-mode-guarded"),
            FilterMode::TangiblesOnly => fl!(language_loader(), "filter-mode-tangibles-only"),
        });
        if let Some(group) = &mode_group {
            btn.set_group(Some(group));
        } else {
            mode_group = Some(btn.clone());
        }
        btn.set_active(mode == current_mode);
        let state_mode = state.clone();
        btn.connect_toggled(move |b| {
            if b.is_active() {
                send_command(
                    &state_mode,
                    ControlCommand::SetFilterMode {
                        mode,
                        stability: None,
                        stage: None,
                    },
                );
            }
        });
        mode_row.append(&btn);
        mode_buttons.push((btn, mode));
    }

    let state_sync = state.clone();
    let mut applied_mode = current_mode;
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let mode = match state_sync.lock() {
            Ok(s) => s.filter_mode,
            Err(_) => return ControlFlow::Continue,
        };
        if mode != applied_mode {
            applied_mode = mode;
            for (btn, button_mode) in &mode_buttons {
                btn.set_active(*button_mode == mode);
            }
        }
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 0);
    body.append(&mode_row);
    body
}
