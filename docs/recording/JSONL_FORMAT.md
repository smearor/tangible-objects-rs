# JSONL File Format

On-disk format of the frame recordings (`<base>.jsonl`) and the exported
labeled datasets (`<stem>.dataset.jsonl`). Rust types are documented in
[`DATA_MODEL.md`](DATA_MODEL.md) — this file is the **wire format**: what is
actually in the bytes.

Both artifacts follow [JSON Lines](https://jsonlines.org): every line is one
complete JSON object, terminated by `\n` (UTF-8, no BOM, no trailing
commas). One line = one event — the stream is append-only within a session
and survives a crash at line granularity.

## `<base>.jsonl` — Frame Records

Written by `FrameRecorder` for every **non-empty** `SYN_REPORT` frame —
frames with neither touches nor tracked tangibles are skipped, leaving gaps
in `frame` (the counter still counts every `SYN_REPORT`, keeping the
numbering aligned with the raw event stream and the device's report rate).

Example line (pretty-printed — on disk it is a single line):

```json
{
  "v": 1,
  "frame": 1024,
  "timestamp_ms": 8534,
  "label": "TestFigur-4P",
  "calibration": {
    "x_min": 0, "x_max": 32767, "y_min": 0, "y_max": 32767,
    "x_res": 300, "y_res": 200, "screen_w_mm": 123.0, "screen_h_mm": 82.0
  },
  "touches": [
    { "slot": 0, "tracking_id": 41, "x": 12.4, "y": 33.1 }
  ],
  "candidates": [
    {
      "pattern_id": 4,
      "pattern_name": "TestFigur-4P",
      "touch_indices": [0, 1, 2, 3],
      "matched": true,
      "partial": false,
      "measured_distances": [33.9, 36.4, 35.0, 44.7, 53.3, 53.5],
      "deviations": [0.6, 0.1, 0.1, 0.1, 0.2, 0.1],
      "tolerance": 3.0,
      "tolerances": [3.0, 2.26, 2.28, 2.33, 2.35, 2.35],
      "score": 0.27,
      "match_error": 1.2
    }
  ],
  "tracked": [
    {
      "id": 7, "pattern_id": 4, "pattern_name": "TestFigur-4P",
      "center": { "x": 61.2, "y": 44.8 }, "rotation_deg": -143.7,
      "frames_since_match": 0
    }
  ],
  "raw_dropped": 0
}
```

### FrameRecord fields

| JSON key          | Type           | Required | Semantics |
|-------------------|----------------|----------|-----------|
| `v`               | `u16`          | always   | Schema version — currently `1`; consumers must refuse unknown versions |
| `frame`           | `u64`          | always   | `SYN_REPORT` ordinal; gaps mark skipped empty frames |
| `timestamp_ms`    | `u64`          | always   | Milliseconds since recording start (replay: source timestamps → deterministic) |
| `label`           | `string\|null` | always   | Ground truth; `null` = unlabeled. Present but nullable — the marker is intentional |
| `calibration`     | `object\|null` | always   | Calibration snapshot; `null` = uncalibrated → `touches` carry raw axis units |
| `touches`         | `array`        | always   | `{slot, tracking_id, x, y}` — positions in millimeters |
| `candidates`      | `array`        | always   | Every tested combination — matched *and* rejected |
| `tracked`         | `array`        | always   | Tracker view incl. `frames_since_match` |
| `raw_dropped`     | `u64` (`EventCount`) | always | Running count of raw events dropped on a full sink buffer — non-zero breaks replay round-trip equality |

### CandidateReport fields

| JSON key             | Type           | Required | Semantics |
|----------------------|----------------|----------|-----------|
| `pattern_id`         | `u32`          | always   | Pattern under test |
| `pattern_name`       | `string`       | always   | Denormalized display name |
| `touch_indices`      | `array<u32>`   | always   | Indices into `touches` — an unordered combination, *not* a pin mapping |
| `matched`            | `bool`         | always   | Passed the tolerance check |
| `partial`            | `bool`         | always   | `num_points − 1` candidate (one pin missing) |
| `measured_distances` | `array<f64>`   | always   | Sorted pairwise distances (mm); index `i` ↔ pattern's `sorted_distances[i]` |
| `deviations`         | `array<f64>`   | always   | `|measured[i] − reference[i]|`, parallel to `measured_distances` |
| `tolerance`          | `f64`          | always   | Scalar tolerance applied (mm) |
| `tolerances`         | `array<f64>`   | optional | Per-index tolerances — **absent** (not `null`) under scalar-only patterns |
| `score`              | `f64`          | optional | Normalized `max_i dev[i]/tol[i]`; absent in older recordings |
| `match_error`        | `f64`          | always   | Sum of `deviations` |

### Reading rules

- **Version gate first**: `v` must equal `RECORD_SCHEMA_VERSION` (1) before
  any field is trusted.
- **Unknown keys are ignored** — newer minor additions stay loadable by
  older readers that tolerate extra fields.
- **`label`/`calibration` are explicit `null` markers** — distinguish
  "absent by old schema" from "recorded as none". Optional candidate fields
  (`tolerances`, `score`) instead *omit* the key entirely.
- Order of lines is frame order; `frame` numbers are monotonically
  increasing.

Quick inspection with `jq`:

```bash
jq 'select(.label != null) | [.frame, .label]' rec.jsonl    # labeled frames
jq '.candidates[] | select(.matched) | .score' rec.jsonl    # matched scores
jq '[.raw_dropped] | max' rec.jsonl                          # must be 0
```

## `<stem>.dataset.jsonl` — Labeled Dataset

Written by `--export-dataset` via `serde_jsonlines` — one `DatasetRow` per
recorded candidate of every **labeled** frame:

```json
{ "frame": 1024, "is_label": true, "matched": true, "partial": false,
  "pattern_id": 4, "score": 0.27, "features": [0.6, 0.04, 0.04, 0.04, 0.09, 0.04] }
```

| JSON key     | Type          | Semantics |
|--------------|---------------|-----------|
| `frame`      | `u64`         | Source frame number |
| `is_label`   | `bool`        | Ground-truth target — candidate's pattern equals the frame label |
| `matched`    | `bool`        | Binary matcher outcome at the recorded tolerance |
| `partial`    | `bool`        | Partial (`num_points − 1`) candidate |
| `pattern_id` | `u32`         | Pattern under test |
| `score`      | `f64`         | `max(features)` — normalized deviation score |
| `features`   | `array<f64>`  | Per-index normalized deviations `dev[i]/tol[i]` — the classifier input |

Empty file when no labeled frames exist — the export reports `0 rows`.

## Companion: `<base>.evtest`

The raw twin artifact is **not** JSON — it is `evtest`-compatible text,
documented in [`../event/EVTEST_FORMAT.md`](../event/EVTEST_FORMAT.md), and
replays through `EvtestReplay`/`--replay`. See
[`../event/DATA_MODEL.md`](../event/DATA_MODEL.md) for the Rust types.
