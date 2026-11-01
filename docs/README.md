# Documentation

Technical documentation for the tangible-objects workspace, organized
by domain. Each folder pairs a `DATA_MODEL.md` (types and invariants)
with topic docs (behavior, formats, workflows). Design rationale lives
in `concepts/{done,planned}/`; these files describe what the code *is*.

## Reading order

New to the project: [`daemon/MODES.md`](daemon/MODES.md) (what the
binary does) → [`interfaces/OVERVIEW.md`](interfaces/OVERVIEW.md) (how
applications talk to it) → the domain docs below.

## By domain

| Domain | Docs |
|--------|------|
| **Daemon** | [`daemon/MODES.md`](daemon/MODES.md) - CLI modes & flags |
| **Events** | [`event/DATA_MODEL.md`](event/DATA_MODEL.md) raw evdev types · [`event/EVTEST_FORMAT.md`](event/EVTEST_FORMAT.md) wire text format · [`event/DOMAIN_EVENTS.md`](event/DOMAIN_EVENTS.md) interface event stream |
| **Device** | [`device/DATA_MODEL.md`](device/DATA_MODEL.md) profiles/registry types · [`device/DETECTION.md`](device/DETECTION.md) enumeration & auto-detect · [`device/CONVERSION.md`](device/CONVERSION.md) cross-device `.evtest` conversion + remote recording |
| **Pattern** | [`pattern/DATA_MODEL.md`](pattern/DATA_MODEL.md) fingerprints & matching · [`pattern/TRACKING.md`](pattern/TRACKING.md) slot affinity, completion, smoothing |
| **Profile** | [`profile/DATA_MODEL.md`](profile/DATA_MODEL.md) profile TOML · [`profile/REGISTRATION.md`](profile/REGISTRATION.md) runtime registration, owners, lifetimes, result types |
| **Fields** | [`fields.md`](fields.md) occupancy field sets |
| **Sets** | [`sets/DATA_MODEL.md`](sets/DATA_MODEL.md) Tangible Object Sets, activation hysteresis, `TOS-*` actions |
| **Filter proxy** | [`proxy/OVERVIEW.md`](proxy/OVERVIEW.md) architecture · [`proxy/SETUP.md`](proxy/SETUP.md) permissions/udev · [`proxy/FILTERING.md`](proxy/FILTERING.md) suppression semantics · [`proxy/PIPELINE.md`](proxy/PIPELINE.md) stage chain & scopes |
| **Recording** | [`recording/DATA_MODEL.md`](recording/DATA_MODEL.md) frame schema + control commands · [`recording/JSONL_FORMAT.md`](recording/JSONL_FORMAT.md) wire format |
| **Analysis** | [`analysis/DATA_MODEL.md`](analysis/DATA_MODEL.md) reports & loaders · [`analysis/METRICS.md`](analysis/METRICS.md) live session metrics |
| **Training** | [`training/WORKFLOW.md`](training/WORKFLOW.md) record → analyze → tune pipeline |
| **Interfaces** | [`interfaces/OVERVIEW.md`](interfaces/OVERVIEW.md) GraphQL/MCP/D-Bus surface |
| **GUI** | [`gui/OVERVIEW.md`](gui/OVERVIEW.md) embedded GTK mode, rotation, alpha, hooks |
| **Clients** | [`clients/CLIENT.md`](clients/CLIENT.md) typed client lib · [`clients/CLI.md`](clients/CLI.md) scriptable CLI · [`clients/TUI.md`](clients/TUI.md) dashboard |
| **i18n** | [`i18n.md`](i18n.md) Fluent localization |

`research/` collects hardware and material research notes (tangibles,
conductive materials, robots) - background, not interface docs.
