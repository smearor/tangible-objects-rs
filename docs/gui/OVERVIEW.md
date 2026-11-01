# Embedded GUI (`--gui`)

Daemon-embedded GTK4 visualization - the crate `tangible-objects-gtk`
(`crates/gtk`), enabled by `--gui` (daemon feature `gui`). Not a remote
client: the engine/device loop runs in-process on a background thread
and the GTK main loop renders the shared state - zero-latency event
flow, fullscreen on the touch surface itself. The embedding daemon
injects its plumbing via [`GuiHooks`](#guihooks) so this crate never
links the daemon crate.

```
crates/gtk/src/
├── lib.rs              - crate docs + run_gui entry point
├── evdev_thread.rs     - in-process engine/device loop thread
├── shared_state.rs     → SharedState   (Mutex'd frame state)
├── gui_options.rs      → GuiOptions    (visible · rotation · alpha)
├── gui_mode.rs         → GuiMode       (Live | Training)
├── gui_hooks.rs        → GuiHooks      (daemon-injected callbacks)
├── background_alpha.rs → BackgroundAlpha (canvas fill opacity)
├── control_bar.rs      - bottom control bar widget
├── draw.rs             - cairo scene rendering
└── i18n.rs             - Fluent loader (i18n/*.ftl)
```

## Runtime structure

```mermaid
flowchart LR
    subgraph GUI["GTK main loop"]
        DA["DrawingArea<br/>cairo draw_func ~30 fps"]
        CB["control bar<br/>overlay"]
        SS["SharedState<br/>(Mutex)"]
        DA --> SS
        CB --> SS
    end
    subgraph EV["evdev thread (in-process)"]
        DEV["/dev/input/eventX"] --> ENG["engine loop<br/>same as --live"]
        ENG --> SS
        HOOKS["GuiHooks<br/>drain commands · interfaces · train"]
        ENG --> HOOKS
    end
    SS -.->|visible/rotation| WIN["window state sync"]
```

- `run_gui` spawns the evdev thread, builds the fullscreen
  `ApplicationWindow` (`DrawingArea` + control bar `Overlay`), and
  calls `app.run_with_args(&["tangible-objects"])` - only argv[0] is
  passed so GTK never sees daemon flags like `--gui`/`--device`
  ("Unbekannte Option").
- A `glib::timeout_add_local` ~30 fps tick queues redraws and syncs
  `SharedState.visible` into the window (`SetGuiVisible` commands).
- **Esc** exits the application (`std::process::exit(0)`).

## GuiHooks (daemon injection)

`run_gui`'s last parameter - the crate boundary. The daemon wires its
own `modes::*` functions in:

| Hook | Daemon function | Purpose |
|------|-----------------|---------|
| `spawn_interfaces` | `modes::spawn_interfaces` | GraphQL/MCP/D-Bus servers on the `EngineHandle` |
| `drain_control_commands` | `modes::drain_control_commands` | applies `ControlCommand`s on the device loop (registration, recording, GUI toggles) |
| `status_of_session` | `modes::status_of_session` | console/status text |
| `effective_calibration` | `modes::effective_calibration` | device-profile resolution override vs. `ABS_INFO` |
| `register_profile` | `modes::register_profile_hook` | save a trained figure from the control bar |

## Rendering state

| Option / command | Type | Effect |
|------------------|------|--------|
| `--display-rotation <0\|90\|180\|270>` | `DisplayRotation` | quarter-turn of evdev positions in the mm→px mapping (`mm_to_screen`) - matches a rotated desktop session (e.g. GNOME portrait on a landscape panel); runtime change via `setDisplayRotation`/`set_display_rotation` |
| `--background-alpha <0-100>` | `BackgroundAlpha` | opacity of the canvas `.surface-bg` CSS layer - lower values let the desktop show through; touches/labels/chrome stay opaque. Control-bar button cycles the presets |
| `SetGuiVisible` command | bool | shows/hides the window without restarting the GUI (`setGuiVisible` / `set_gui_visible`) |
| `showMessage` / `ShowMessage` | string | large centered overlay text; empty clears - scripted training instructions ([`../analysis/METRICS.md`](../analysis/METRICS.md)) |

The window background itself is transparent (`window { background-color:
transparent }` at application priority) so the canvas alpha actually
reveals the desktop beneath; pending partial-spawn tangibles render
dimmed ([`../pattern/TRACKING.md`](../pattern/TRACKING.md)).

## Control bar

Bottom overlay: **mode toggle** (live ↔ training), **name entry**,
**save-profile** button (writes `profiles/<name>.toml` via the
`register_profile` hook), **alpha cycle** button, status label. All
strings come from Fluent (`i18n/{de,en}/tangible-objects-gtk.ftl` -
[`../i18n.md`](../i18n.md)).

## Planned

Wayland layer-shell surfaces (`overlay`/`top`/`bottom` layers via
`gtk4-layer-shell`), per-window anchors and rotated input regions -
`concepts/planned/GUI.md` §7. The system dependency
`libgtk4-layer-shell-dev` is already part of the CI image list.
