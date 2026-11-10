//! Daemon-embedded GTK visualization (`concepts/planned/GUI.md` +
//! WORKSPACE.md Phase 3).
//!
//! Embedded library, not a remote client: the embedding daemon calls
//! [`run_gui`], which runs the engine/device loop on a background
//! thread (in-process, zero-latency event flow) and renders the
//! [`SharedState`] in the GTK main loop. Daemon-mode semantics
//! (interfaces, control commands, recording, calibration) arrive via
//! [`GuiHooks`] so this crate never links the daemon crate.

mod background_alpha;
mod calibration_card;
mod calibration_source;
mod card_layout;
mod dead_zones_card;
mod device_card;
mod draw;
mod evdev_thread;
mod field_set_row;
mod field_sets_card;
mod filter_card;
mod gui_hooks;
mod gui_mode;
mod gui_options;
mod i18n;
mod info_card;
mod metrics_card;
mod parameters_card;
mod passthrough;
mod pattern_style;
mod profiles_card;
mod shared_state;
mod touches_card;
mod training_card;
mod visualization_card;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use floating_board::BoundsMode;
use floating_board::CardAnchor;
use floating_board::CardConfig;
use floating_board::CardId;
use floating_board::FloatingBoard;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::DrawingArea;
use gtk4::glib::ControlFlow;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use tangible_objects_model::DetectionConfig;
use tangible_objects_model::PatternBuf;
use tracing::info;
use tracing::warn;

pub use background_alpha::BackgroundAlpha;
pub use gui_hooks::GuiHooks;
pub use gui_mode::GuiMode;
pub use gui_options::GuiOptions;
pub use i18n::language_loader;
pub use shared_state::SharedState;

/// Fullscreen GTK visualization with control bar and evdev-driven
/// drawing. `patterns` is pre-loaded by the daemon (`load_patterns`);
/// `visible` carries the initial window state (`--gui` shows by
/// default; `SetGuiVisible` commands toggle afterwards).
pub fn run_gui(
    device_path: String,
    profiles_dir: Option<String>,
    patterns: PatternBuf,
    device_profiles: Option<PathBuf>,
    config: DetectionConfig,
    options: GuiOptions,
    hooks: GuiHooks,
) {
    let state = SharedState::new(&options);

    // Spawn evdev thread - the in-process engine loop.
    let state_clone = state.clone();
    let device_profiles_arc = device_profiles.map(Arc::new);
    let device_path_thread = device_path.clone();
    std::thread::spawn(move || {
        evdev_thread::evdev_thread(
            device_path_thread,
            patterns,
            state_clone,
            config,
            options.rotation,
            device_profiles_arc,
            hooks,
        );
    });

    let app = Application::builder()
        .application_id("dev.aschaeffer.tangible-objects")
        .build();

    let state_app = state.clone();
    let profiles_dir_app = profiles_dir.clone();
    let device_path_app = device_path.clone();
    let visible_app = options.visible;
    app.connect_activate(move |app| {
        let drawing_area = DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);

        // The canvas darkening lives in the widget's CSS background
        // layer (`.surface-bg`) - its alpha blends against the
        // transparent window, so the toggle sees the desktop beneath.
        // The cairo draw stays content-only on top of it.
        let bg_provider = gtk4::CssProvider::new();
        bg_provider.load_from_string(&options.background_alpha.surface_bg_css());
        drawing_area.add_css_class("surface-bg");
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &bg_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let state_draw = state_app.clone();
        drawing_area.set_draw_func(move |_, cr, w, h| {
            draw::draw_scene(cr, w, h, &state_draw);
        });

        // Layout: the drawing area fills the window; all controls live
        // on floating cards.
        let overlay = gtk4::Overlay::new();
        overlay.set_child(Some(&drawing_area));

        // Canvas redraw cadence - the evdev thread writes
        // `SharedState` per frame; this tick repaints at ~10 Hz.
        let da_tick = drawing_area.clone();
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            da_tick.queue_draw();
            gtk4::glib::ControlFlow::Continue
        });

        // Free-floating cards on top of the surface view
        // (concepts/planned/GUI.md §3): the board stacks a transparent
        // card layer above the overlay; gestures landing outside a
        // card fall through to the canvas.
        let board = FloatingBoard::new(&overlay);
        let info_body = info_card::create_info_card(&state_app);
        let _ = board.add_card_with_id(
            "info",
            &info_body,
            CardConfig {
                title: Some(fl!(language_loader(), "info-card-title")),
                position: CardAnchor::TopRight,
                anchor_margin: 100.0,
                bounds_mode: BoundsMode::Contained,
                foldable: true,
                ..Default::default()
            },
        );
        for (id, body, title, anchor) in [
            (
                "visualization",
                visualization_card::create_visualization_card(
                    &state_app,
                    &drawing_area,
                    &bg_provider,
                ),
                fl!(language_loader(), "card-visualization-title"),
                CardAnchor::TopLeft,
            ),
            (
                "filter",
                filter_card::create_filter_card(&state_app),
                fl!(language_loader(), "card-filter-title"),
                CardAnchor::CenterLeft,
            ),
            (
                "device",
                device_card::create_device_card(&state_app, &device_path_app),
                fl!(language_loader(), "card-device-title"),
                CardAnchor::BottomCenter,
            ),
            (
                "parameters",
                parameters_card::create_parameters_card(&state_app),
                fl!(language_loader(), "card-parameters-title"),
                CardAnchor::BottomRight,
            ),
            (
                "touches",
                touches_card::create_touches_card(&state_app),
                fl!(language_loader(), "card-touches-title"),
                CardAnchor::BottomCenter,
            ),
            (
                "calibration",
                calibration_card::create_calibration_card(&state_app),
                fl!(language_loader(), "card-calibration-title"),
                CardAnchor::Center,
            ),
            (
                "metrics",
                metrics_card::create_metrics_card(&state_app),
                fl!(language_loader(), "card-metrics-title"),
                CardAnchor::CenterRight,
            ),
            (
                "profiles",
                profiles_card::create_profiles_card(&state_app),
                fl!(language_loader(), "card-profiles-title"),
                CardAnchor::CenterRight,
            ),
            (
                "field-sets",
                field_sets_card::create_field_sets_card(&state_app),
                fl!(language_loader(), "card-field-sets-title"),
                CardAnchor::TopCenter,
            ),
            (
                "dead-zones",
                dead_zones_card::create_dead_zones_card(&state_app),
                fl!(language_loader(), "card-dead-zones-title"),
                CardAnchor::BottomLeft,
            ),
        ] {
            let _ = board.add_card_with_id(
                id,
                &body,
                CardConfig {
                    title: Some(title),
                    position: anchor,
                    anchor_margin: 100.0,
                    bounds_mode: BoundsMode::Contained,
                    foldable: true,
                    ..Default::default()
                },
            );
        }

        // Card layout: restore `gui-cards.toml` (or arrange the
        // default side-by-side), persist on geometry changes.
        let layout = card_layout::load_layout();
        // The training card exists only in training mode - its saved
        // placement (if any) seeds the add/remove watcher.
        let training_placement = layout.as_ref().and_then(|layout| {
            layout
                .cards
                .iter()
                .find(|card| card.id == CardId::new("training"))
                .cloned()
        });
        card_layout::wire_layout(&board, layout);
        training_card::wire_training_card(
            &board,
            &state_app,
            &profiles_dir_app,
            training_placement,
        );

        // Touch passthrough: input reaches the cards; everything else
        // falls through to the desktop.
        passthrough::wire_passthrough(&board, &state_app);

        let window = ApplicationWindow::builder()
            .application(app)
            .title(fl!(language_loader(), "window-title"))
            .default_width(1600)
            .default_height(1200)
            .child(&board)
            .build();

        // `--touch-passthrough` puts the window into the layer-shell
        // Overlay layer: a toplevel loses stacking when the
        // through-click focuses the window beneath (GNOME), and GTK4
        // has no `keep_above`. Must run before `present`; runtime
        // toggles only adjust the input region. `init_layer_shell`
        // degrades to a regular toplevel when the protocol is missing
        // - logging makes the fallback visible on remote machines.
        if options.touch_passthrough {
            use gtk4_layer_shell::Edge;
            use gtk4_layer_shell::KeyboardMode;
            use gtk4_layer_shell::Layer;
            use gtk4_layer_shell::LayerShell;

            if gtk4_layer_shell::is_supported() {
                info!("layer-shell overlay enabled (touch passthrough)");
            } else {
                warn!("layer-shell not supported - window stays a regular toplevel");
            }

            window.init_layer_shell();
            window.set_layer(Layer::Overlay);
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Right, true);
            // -1 = ignore other layer surfaces' exclusive zones - the
            // surface spans edge-to-edge over bars/docks reserved by
            // other apps (e.g. waybar on the same Overlay layer).
            window.set_exclusive_zone(-1);
            // Stable namespace so compositor rules can match the
            // surface (e.g. Hyprland `layerrule`).
            window.set_namespace(Some("tangible-objects"));
            window.set_keyboard_mode(KeyboardMode::None);
        }

        // Window background transparent so the canvas fill's alpha
        // (`background_alpha` toggle) actually shows the desktop
        // beneath - the theme paints the window background opaque by
        // default, covering the translucent cairo fill. Only the
        // window *background* is cleared; contents stay opaque.
        let provider = gtk4::CssProvider::new();
        provider.load_from_string("window { background-color: transparent; }");
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        window.fullscreen();
        window.set_visible(visible_app);

        // Redraw timer ~30fps + visibility sync (SetGuiVisible).
        let state_tick = state_app.clone();
        let da_tick = drawing_area.clone();
        let win_tick = window.clone();
        gtk4::glib::timeout_add_local(Duration::from_millis(33), move || {
            if let Ok(s) = state_tick.lock()
                && win_tick.is_visible() != s.visible
            {
                win_tick.set_visible(s.visible);
            }
            da_tick.queue_draw();
            ControlFlow::Continue
        });

        // Escape exits fullscreen.
        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(|_, keyval, _, _| {
            if keyval == gtk4::gdk::Key::Escape {
                std::process::exit(0);
            }
            Propagation::Proceed
        });
        window.add_controller(key_controller);
    });

    // `run()` would hand `std::env::args` to `g_application_run` - GTK
    // would reject daemon flags like `--gui`/`--device` as unknown
    // options ("Unbekannte Option"), so only argv[0] is passed.
    app.run_with_args(&["tangible-objects"]);
}
