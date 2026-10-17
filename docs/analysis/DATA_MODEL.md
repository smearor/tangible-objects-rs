# Analysis Data Model

Offline evaluation of recordings - the crate `tangible-objects-analysis`
(`crates/analysis`). It loads `*.jsonl` recordings ([frame schema
`"v": 1`](../recording/DATA_MODEL.md)) into typed `FrameRecord`s - the same
types that serialize in `model`, so writer and reader cannot drift apart -
and renders human-readable reports for TRAINING.md phases 2 and 3.

```
crates/analysis/src/
├── lib.rs               - crate docs + re-exports
├── error.rs             → AnalysisError
├── recording.rs         → Recording           (JSONL loader + version gate)
├── percentile.rs        → Percentiles         (generic over Into<f64>)
├── utilization/
│   ├── mod.rs, accumulator.rs
│   ├── pattern.rs       → PatternUtilization
│   └── report.rs        → UtilizationReport
├── tolerance/
│   ├── mod.rs
│   ├── suggestion.rs    → ToleranceSuggestion
│   └── report.rs        → ToleranceReport, DEFAULT_SAFETY_FACTOR (1.25)
├── flappy/
│   ├── mod.rs
│   ├── bucket.rs        → LifetimeBucket
│   ├── partial_share.rs → PartialShare
│   └── report.rs        → FlappyReport
├── collision/
│   ├── mod.rs, accumulator.rs
│   ├── cell.rs          → CollisionCell
│   └── report.rs        → CollisionReport
├── scoring/
│   ├── mod.rs
│   ├── row.rs           → ThresholdRow
│   └── report.rs        → ScoringReport
├── ml/
│   ├── mod.rs
│   ├── accuracy.rs      → Accuracy            (clamped 0..=1)
│   ├── classifier.rs    → LogisticBaseline    (dependency-free)
│   └── report.rs        → MlReport
└── dataset/
    ├── mod.rs
    ├── row.rs           → DatasetRow
    └── jsonl.rs         → Dataset             (serde_jsonlines export)
```

Invoked via `tangible-objects --analyze <recording.jsonl>` and
`--export-dataset <recording.jsonl>` (`crates/daemon/src/modes.rs`).

## Loading (`Recording`)

`Recording::load(path)` parses one `FrameRecord` per line, checks
`schema_version == RECORD_SCHEMA_VERSION` per line (`UnsupportedVersion`
with the 1-based line number) and rejects malformed JSON (`Parse`).
Unknown fields are tolerated - older/newer minor shapes stay loadable;
new optional record fields (`score`, `tolerances`) default to absent.

## Reports

All reports implement `Display` for the `--analyze` CLI output and are pure
functions of `&Recording` - no engine state.

| Report               | Phase | Question answered |
|----------------------|-------|-------------------|
| `UtilizationReport`  | 2.1   | How much of the configured tolerance does each distance index consume? p50/p95/p99/max per index, utilization bands, false negatives via labels |
| `ToleranceReport`    | 2.2   | Which `tolerances[]` does the data justify? `p99[i] × 1.25` per index, conservative scalar fallback, TOML snippet |
| `FlappyReport`       | 2.3   | How stable are contacts? Tracking-id lifetime histogram, dropout gaps (`frames_since_match` runs), partial-match share |
| `CollisionReport`    | 2.4   | Which foreign patterns explain labeled geometry? Per label × pattern: collided frames + required-tolerance percentiles |
| `ScoringReport`      | 3.2   | Where does the acceptance threshold sit? τ-sweep (0.5–2.0) with recall/precision, own-vs-foreign score separation |
| `MlReport`           | 3.3   | Does a learned classifier beat the score rule? `LogisticBaseline` vs. `score ≤ 1.0` on a deterministic hold-out |
| `Dataset`            | 3.3   | JSONL export of labeled candidates (`features` = normalized deviations, `is_label` target) |

`UtilizationReport` keys patterns by `PatternId` (stable identity); the
name is display-only. Label-only rows - a label that never produced a
candidate - appear with `pattern_id: None`.

## Types

| Type          | Wraps   | Purpose |
|---------------|---------|---------|
| `Percentiles` | `f64`×6 | min/p50/p95/p99/max/mean; `of(Iterator<Item = impl Into<f64>>)` accepts `Millimeters`/`MatchScore`/`f64` uniformly - the unit crosses only at the stats boundary |
| `Accuracy`    | `f64`   | Clamped `0.0..=1.0`; `Option<Accuracy>` = not evaluable |
| `DatasetRow`  | -       | Serialized per line: `frame` `is_label` `matched` `partial` `pattern_id` `score` `features[]` |

## Design Rules

- **Shared schema**: records deserialize with the `model` types - no
  mirrored struct definitions.
- **Newtypes at boundaries**: accumulators count in `FrameCount`/
  `CandidateCount`, samples stay `Millimeters`/`MatchScore`; `f64` is the
  working representation inside `Percentiles` only.
- **Labels are ground truth**: utilization false negatives, the collision
  matrix, the score sweep and ML evaluation all require labeled frames;
  unlabeled recordings print a hint instead of empty tables.
- **No per-pin attribution**: `touch_indices` is an unordered combination -
  the flappy report measures per-contact stability, not per-pin.
