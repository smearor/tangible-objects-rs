# Daemon Modes & CLI Reference

The `tangible-objects` binary (`crates/daemon`) is mode-driven: exactly
one mode flag selects the action, all other options configure it
(`clap` group `mode`). `--record` is ambiguous by design: alone it is a
mode of its own, next to `--live`/`--replay` a modifier.

```bash
tangible-objects --<mode> [options]
```

## Modes

| Mode | Device? | What it does |
|------|---------|--------------|
| `--live` | ✓ | Live detection from an evdev device; console line per frame |
| `--record <base>` | ✓ | Live + recording: `<base>.jsonl` frames + `<base>.evtest` raw stream |
| `--replay --file <evtest>` | – | Deterministic replay of a `.evtest` artifact (hardware-free); combine with `--record` to re-record |
| `--filter` | ✓ | Filter proxy: `EVIOCGRAB` + uinput clone, tangibles suppressed ([`../proxy/OVERVIEW.md`](../proxy/OVERVIEW.md)); `--record <base>` modifier records both sides: `<base>.evtest` input + `<base>.out.evtest` emitted stream |
| `--train --name <n>` | ✓ | Capture a figure's pin geometry into `profiles/<n>.toml` |
| `--gui` | ✓ | Daemon-embedded GTK fullscreen visualization, feature-gated `gui` ([`../gui/OVERVIEW.md`](../gui/OVERVIEW.md)) |
| `--list` | – | Enumerate `/dev/input/event*` with vendor:product IDs |
| `--analyze <rec.jsonl>` | – | Offline reports: utilization, deviations, tolerances, flappy, collisions, score sweep, ML baseline |
| `--similarity <profiles-dir>` | – | Profile-collision surface analysis; `--similarity-json <file>` exports the report |
| `--export-dataset <rec.jsonl>` | – | Labeled candidate features → `<stem>.dataset.jsonl` |
| `--convert-device <in.evtest> --to <p\|file>` | – | Re-emit a recording in another `DeviceProfile`'s raw space |

Device-less modes (`--analyze`, `--similarity`, `--export-dataset`,
`--convert-device`, `--list`) never touch `/dev/input`. `--live`,
`--record`, `--filter`, `--train` auto-detect the touchscreen via the
device registry when `--device` is omitted
([`../device/DETECTION.md`](../device/DETECTION.md)); `--gui` requires an
explicit `--device`.

## Common options

| Option | Value | Applies to |
|--------|-------|------------|
| `--device <path>` | `/dev/input/eventX` | device modes (auto-detect otherwise) |
| `--profiles <dir>` | dir | pattern registry (default `profiles/`) |
| `--device-profiles <dir>` | dir | extra `DeviceProfile` TOMLs merged after builtins |
| `--name <n>` | string | `--train` figure name |
| `--tolerance <mm>` | f64 | `--train` matching tolerance (default 30.0) |
| `--record-label <s>` | string | ground-truth label on `--record` |
| `--dead-zones-visible` | bool | draws the dead-zone overlay on the GUI canvas (`--gui`) |
| `--no-import-slots` | flag | skip importing kernel MT slot state at startup (contacts opened before the daemon stay invisible) |
| `--to <p\|file.toml>` | name or path | `--convert-device` target profile |

## Interfaces

| Option | Value | Notes |
|--------|-------|-------|
| `--graphql-listen <addr>` | `127.0.0.1:8080` | GraphQL endpoint incl. GraphiQL on `GET /` |
| `--mcp-listen <addr>` | `127.0.0.1:8090` | MCP streamable HTTP on `/mcp` |
| `--dbus` | flag | `org.tangibleobjects.Engine1` on the session bus |

All three may run in parallel ([`../interfaces/OVERVIEW.md`](../interfaces/OVERVIEW.md)).

## Filter & detection tuning

| Option | Value | Notes |
|--------|-------|-------|
| `--legacy <auto\|always\|never>` | enum | `BTN_TOUCH`/`ABS_X/Y` policy ([`../proxy/FILTERING.md`](../proxy/FILTERING.md)) |
| `--stability-frames <n>` | u64 | suppression gate, default 2 (gated/guarded) |
| `--edge-filter-mm <mm>` | f64 | drop ghost contacts near panel borders (EdgeStage) |
| `--zone <x0,y0,x1,y1>` | mm rect | masked zone, repeatable (ZoneStage) |
| `--fields <file\|dir>` | path | occupancy field sets ([`../fields.md`](../fields.md)) |

## Completion & tracking

Overrides for `~/.config/tangible-objects/completion.toml`
([`../pattern/TRACKING.md`](../pattern/TRACKING.md)):

| Option | Value | Default |
|--------|-------|---------|
| `--completion-mode <match\|exact>` | enum | `match` |
| `--completion-radius <f>` | f64 | 2.0 (× tolerance) |
| `--completion-max-missing <1\|2>` | u32 | 1 |
| `--completion-no-persist` | flag | claimed slots stay unbound |
| `--affinity-factor <f>` | f64 | 2.0 (`1.0` = off) |
| `--smoothing-alpha <a>` | f64 | 0.6 (`0` = off) |

## GUI

| Option | Value | Notes |
|--------|-------|-------|
| `--display-rotation <0\|90\|180\|270>` | deg | quarter-turn applied to evdev positions before rendering - for rotated desktop sessions |
| `--background-alpha <0-100>` | percent | canvas fill opacity; 100 = opaque |

## During a session

- **stdin** (`--record`): type a label + Enter to tag subsequent frames;
  empty line/`q` stops cleanly.
- **console**: `Frame N | touches | figures | suppressed | forwarded |
  sets | fields | occupied` (filter mode).
- **Esc** exits the fullscreen GUI (`--gui`).
