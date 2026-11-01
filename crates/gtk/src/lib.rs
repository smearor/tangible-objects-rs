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
mod control_bar;
mod draw;
mod evdev_thread;
mod gui_hooks;
mod gui_mode;
mod gui_options;
mod i18n;
mod shared_state;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::DrawingArea;
use gtk4::glib::ControlFlow;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use i18n_embed_fl::fl;
use tangible_objects_model::DetectionConfig;
use tangible_objects_model::PatternBuf;

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
    let state = SharedState::new(options.visible, options.rotation, options.background_alpha);

    // Spawn evdev thread - the in-process engine loop.
    let state_clone = state.clone();
    let device_profiles_arc = device_profiles.map(Arc::new);
    std::thread::spawn(move || {
        evdev_thread::evdev_thread(
            device_path,
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
        bg_provider.load_from_data(&options.background_alpha.surface_bg_css());
        drawing_area.add_css_class("surface-bg");
        drawing_area
            .style_context()
            .add_provider(&bg_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let state_draw = state_app.clone();
        drawing_area.set_draw_func(move |_, cr, w, h| {
            draw::draw_scene(cr, w, h, &state_draw);
        });

        // Control bar.
        let control_bar = control_bar::create_control_bar(
            &state_app,
            &profiles_dir_app,
            &drawing_area,
            &bg_provider,
        );

        // Layout: drawing area + control bar overlay.
        let overlay = gtk4::Overlay::new();
        overlay.set_child(Some(&drawing_area));
        overlay.add_overlay(&control_bar);

        let window = ApplicationWindow::builder()
            .application(app)
            .title(fl!(language_loader(), "window-title"))
            .default_width(1600)
            .default_height(1200)
            .child(&overlay)
            .build();

        // Window background transparent so the canvas fill's alpha
        // (`background_alpha` toggle) actually shows the desktop
        // beneath - the theme paints the window background opaque by
        // default, covering the translucent cairo fill. Only the
        // window *background* is cleared; contents stay opaque.
        let provider = gtk4::CssProvider::new();
        provider.load_from_data("window { background-color: transparent; }");
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
