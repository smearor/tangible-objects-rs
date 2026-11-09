//! The visualization card - the canvas overlay toggles (edge margin,
//! fields, ruler, grid, touch passthrough) as a movable card on the
//! `FloatingBoard` (concepts/planned/GUI.md §3).

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use gtk4::Box as GtkBox;
use gtk4::Button;
use gtk4::DrawingArea;
use gtk4::Orientation;
use gtk4::Separator;
use gtk4::ToggleButton;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use tangible_objects_runtime::ControlCommand;

use crate::language_loader;
use crate::shared_state::SharedState;
use crate::shared_state::send_command;

/// `SharedState` flag reader - the overlay toggles sync their
/// checked state from it.
type OverlayRead = fn(&SharedState) -> bool;

/// Builds the visualization-card content: one toggle per canvas
/// overlay plus the background-alpha cycler, synced back from
/// [`SharedState`] every 100 ms - interface mutations
/// (GraphQL/D-Bus/MCP) flip the buttons, button clicks send
/// `ControlCommand`s over the same channel.
pub fn create_visualization_card(
    state: &Arc<Mutex<SharedState>>,
    drawing_area: &DrawingArea,
    bg_provider: &gtk4::CssProvider,
) -> GtkBox {
    let toggles = GtkBox::new(Orientation::Vertical, 4);

    let mut overlay_buttons: Vec<(ToggleButton, OverlayRead)> = Vec::new();
    let mut add_overlay_toggle =
        |label: &str, read: OverlayRead, command: fn(bool) -> ControlCommand| {
            let btn = ToggleButton::with_label(label);
            btn.set_active(state.lock().map(|s| read(&s)).unwrap_or_default());
            let state_click = state.clone();
            btn.connect_toggled(move |b| {
                send_command(&state_click, command(b.is_active()));
            });
            toggles.append(&btn);
            overlay_buttons.push((btn, read));
        };
    add_overlay_toggle(
        &fl!(language_loader(), "toggle-edge-margin"),
        |s| s.edge_margin_visible,
        ControlCommand::SetEdgeMarginVisible,
    );
    add_overlay_toggle(
        &fl!(language_loader(), "toggle-fields"),
        |s| s.fields_visible,
        ControlCommand::SetFieldsVisible,
    );
    add_overlay_toggle(
        &fl!(language_loader(), "toggle-ruler"),
        |s| s.ruler_visible,
        ControlCommand::SetRulerVisible,
    );
    add_overlay_toggle(
        &fl!(language_loader(), "toggle-grid"),
        |s| s.grid_visible,
        ControlCommand::SetGridVisible,
    );
    add_overlay_toggle(
        &fl!(language_loader(), "toggle-positions"),
        |s| s.positions_visible,
        ControlCommand::SetPositionsVisible,
    );
    add_overlay_toggle(
        &fl!(language_loader(), "toggle-passthrough"),
        |s| s.touch_passthrough,
        ControlCommand::SetTouchPassthrough,
    );

    toggles.append(&Separator::new(Orientation::Horizontal));

    // Alpha cycler - rotates the background opacity presets; touches,
    // labels and chrome stay opaque, only the canvas fill thins.
    let alpha_btn = Button::with_label(&fl!(
        language_loader(),
        "background-alpha",
        percent = (state
            .lock()
            .map(|s| s.background_alpha.percent() as i64)
            .unwrap_or(100))
    ));
    let state_alpha = state.clone();
    let alpha_btn_click = alpha_btn.clone();
    let provider_alpha = bg_provider.clone();
    let da_alpha = drawing_area.clone();
    alpha_btn.connect_clicked(move |_| {
        let Ok(mut s) = state_alpha.lock() else {
            return;
        };
        s.background_alpha = s.background_alpha.next();
        s.background_alpha_request = Some(s.background_alpha);
        alpha_btn_click.set_label(&fl!(
            language_loader(),
            "background-alpha",
            percent = (s.background_alpha.percent() as i64)
        ));
        provider_alpha.load_from_data(&s.background_alpha.surface_bg_css());
        da_alpha.queue_draw();
    });
    toggles.append(&alpha_btn);

    let state_sync = state.clone();
    let mut applied = state_sync
        .lock()
        .map(|s| {
            overlay_buttons
                .iter()
                .map(|(_, read)| read(&s))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let provider_sync = bg_provider.clone();
    let alpha_btn_sync = alpha_btn.clone();
    let mut applied_alpha = state_sync
        .lock()
        .map(|s| s.background_alpha)
        .unwrap_or_else(|_| crate::BackgroundAlpha::from_percent(100));
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_sync.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        // Alpha changes from `SetBackgroundAlpha` commands land in the
        // shared state via the device loop - the provider reload and
        // button label follow here (the click applies both itself).
        if s.background_alpha != applied_alpha {
            applied_alpha = s.background_alpha;
            provider_sync.load_from_data(&applied_alpha.surface_bg_css());
            alpha_btn_sync.set_label(&fl!(
                language_loader(),
                "background-alpha",
                percent = (applied_alpha.percent() as i64)
            ));
        }
        // Overlay changes from interface commands land in shared
        // state via the device loop - sync the buttons (set_active
        // re-fires toggled and re-sends an idempotent command).
        for ((btn, read), applied) in overlay_buttons.iter().zip(applied.iter_mut()) {
            let current = read(&s);
            if current != *applied {
                *applied = current;
                btn.set_active(current);
            }
        }
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 0);
    body.append(&toggles);
    body
}
