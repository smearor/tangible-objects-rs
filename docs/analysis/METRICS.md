# Session Metrics: Live Recognition Quality Counters

The daemon accumulates recognition metrics on every processed frame -
the same numbers previously extracted ad-hoc from `.jsonl` recordings.
They are readable live over all interfaces and resettable between test
runs, enabling automated test series.

## Data Model

`SessionMetrics` is published inside every `EngineSnapshot` and defined
in `model::metrics`:

```rust
pub struct SessionMetrics {
    pub frames: FrameCount,          // total processed frames
    pub tracked_frames: FrameCount,  // frames with ≥1 tracked tangible
    pub contacts: ContactCount,      // total touch contacts observed
    pub active_frames: FrameCount,   // frames with TouchPointsUpdated
    pub slot_churn: HashMap<TouchPointSlot, ContactCount>,
    pub patterns: HashMap<PatternName, PatternMetrics>,
    pub tangible_count: TangibleCount,  // distinct identities ever tracked
}

pub struct PatternMetrics {
    pub tracked_frames: FrameCount,
    pub tangible_count: TangibleCount,      // re-ID churn for this pattern
    pub exact_match_frames: FrameCount,
    pub partial_match_frames: FrameCount,
    pub claimed_pins: ContactCount,
    pub max_match_deviation: Option<Millimeters>,
}
```

### Reading the numbers

| Metric | Meaning |
|---|---|
| `tracked_frames / frames` | Uptime - how long anything was recognized |
| `tangible_count` | Re-ID churn: how often the same physical figure expired and re-spawned. `1` = never re-identified |
| `patterns[name]` | Counters split per pattern *name* - `"FM4"` vs `"FM4 (partial)"` stay separate, so a figure living in partial mode is visible |
| `patterns[].tangible_count` | Per-figure churn - the FM4-vs-TestFigur comparison in one field |
| `exact` vs `partial_match_frames` | How often the figure matched fully vs. needed a completion claim |
| `claimed_pins` | Missing-pin completion activity |
| `slot_churn[slot]` | Distinct tracking IDs per slot - counts physical pin flapping; `1` = rock-solid contact |
| `active_frames` | Frames the touch set actually changed |

`metrics` is cumulative from daemon start (or the last `ResetMetrics`).
Frames where no touch changes happened still count toward `frames` only
when the engine actually evaluated them.

## Interfaces

| Interface | Read | Reset |
|---|---|---|
| GraphQL | `metrics` (JSON) | `mutation { resetMetrics }` |
| MCP | `get_metrics` | `reset_metrics` |
| D-Bus | `metrics` (JSON string) | `reset_metrics` |

Reset semantics: counters drop to zero from the next frame on; the
snapshot publishes the emptied metrics immediately.

## Automated Test Series

The combination `show_message` + `reset_metrics` + `metrics` scripts a
full series without downloading artifacts:

```
show_message  "Legen Sie Holzklotz2 auf"
  ... user places the figure ...
reset_metrics
  ... user moves the figure for N seconds ...
metrics       -> uptime, churn, exact/partial split, slot flapping
```

For GUI-mode daemons `show_message` renders the instruction as a large
centered overlay text; headless modes print it to stdout. An empty text
clears the overlay.

## Metrics vs. Recordings

| Question | Tool |
|---|---|
| "How stable is figure X during motion?" | `metrics` - uptime, churn, match split |
| "*Why* did the match fail?" (whole recording) | `DeviationReport` - `--analyze` or MCP `analyze_recording` |
| "*Why* did the match fail at frame 641 specifically?" | `.jsonl` recording - `candidates[].deviations` |
| Test-series comparison (config A vs B) | `metrics` after `reset_metrics` |

`SessionMetrics` aggregates *counts*; the causal per-distance analysis
is automated by `analysis::DeviationReport` (see below) over a
recording's `candidates` diagnostics (see
`docs/recording/JSONL_FORMAT.md`). Both layers complement each other:
metrics quantify stability, deviation analysis explains failures.

## Deviation Analysis (automated)

`DeviationReport` folds every *full* candidate of every pattern into
per-distance-index statistics - the Rust equivalent of manually
inspecting `candidates[].deviations` around dropout windows:

| Field | Meaning |
|---|---|
| `distances[i].mean / max / p99` | Deviation of fingerprint distance `i` (a concrete pin pair) |
| `over_tolerance` | Frames where distance `i` exceeded its effective tolerance (per-index `tolerances[i]` if configured, else scalar) |
| `blocker` | Frames where distance `i` was *among* the over-tolerance distances |
| `sole_blocker` | Frames where distance `i` was the **only** over-tolerance distance - the direct cause of the miss |

`patterns[].matched_frames`/`missed_frames` count full-candidate
outcomes; `missed_frames` + the blocker histogram answer "which pin
pair keeps killing the match".

Available as `tangible-objects --analyze <recording.jsonl>` (prints
the section "Abweichungs-Analyse") and as MCP tool `analyze_recording`
(returning the report as JSON).
