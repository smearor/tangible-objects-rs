# Analysis Data Model

Offline evaluation of recordings - the crate `tangible-objects-analysis`
(`crates/analysis`). It loads `*.jsonl` recordings ([frame schema
`"v": 1`](../recording/DATA_MODEL.md)) into typed `FrameRecord`s - the same
types that serialize in `model`, so writer and reader cannot drift apart -
and renders human-readable reports for TRAINING.md phases 2 and 3.

```
crates/analysis/src/
├── lib.rs                - crate docs + re-exports
├── error.rs              → AnalysisError
├── recording.rs          → Recording           (JSONL loader + version gate)
├── percentile.rs         → Percentiles         (generic over Into<f64>)
├── deviation_samples.rs  → DeviationSamples    (per-index sample column)
├── utilization/
│   ├── mod.rs, accumulator.rs
│   ├── pattern.rs        → PatternUtilization
│   ├── ratio_bands.rs    → RatioBands          (named quartile fields + Display)
│   └── report.rs         → UtilizationReport
├── deviation/
│   ├── mod.rs, accumulator.rs → PatternAcc     (per-index accumulators)
│   ├── best_subset.rs    → BestSubset          (best-fitting subset per frame)
│   ├── distance_stats.rs → DistanceStats
│   ├── pattern_deviation.rs → PatternDeviation
│   └── report.rs         → DeviationReport     (Candidates + Contacts modes)
├── tolerance/
│   ├── mod.rs
│   ├── suggestion.rs     → ToleranceSuggestion
│   └── report.rs         → ToleranceReport, DEFAULT_SAFETY_FACTOR (1.25)
├── flappy/
│   ├── mod.rs
│   ├── bucket.rs         → LifetimeBucket
│   ├── partial_share.rs  → PartialShare
│   └── report.rs         → FlappyReport
├── collision/
│   ├── mod.rs, accumulator.rs
│   ├── cell.rs           → CollisionCell
│   └── report.rs         → CollisionReport
├── similarity/
│   ├── mod.rs
│   ├── match_risk.rs     → MatchRisk           (Exact | Partial | Safe)
│   ├── pair.rs           → SimilarityPair      (closest() subset sweep)
│   ├── subset_collision.rs → SubsetCollision
│   └── report.rs         → SimilarityReport    (profile-dir phantom surface)
├── scoring/
│   ├── mod.rs
│   ├── row.rs            → ThresholdRow
│   └── report.rs         → ScoringReport
├── ml/
│   ├── mod.rs
│   ├── accuracy.rs       → Accuracy            (clamped 0..=1)
│   ├── classifier.rs     → LogisticBaseline    (dependency-free)
│   └── report.rs         → MlReport
├── session/
│   ├── mod.rs
│   └── report.rs         → AnalysisReport      (all reports, one bundle)
└── dataset/
    ├── mod.rs
    ├── row.rs            → DatasetRow
    └── jsonl.rs          → Dataset             (serde_jsonlines export)
```

Invoked via `tangible-objects --analyze <recording.jsonl>`,
`--export-dataset <recording.jsonl>` and `--similarity <profiles-dir>`
(`crates/daemon/src/modes/analysis.rs`).

## Loading (`Recording`)

`Recording::load(path)` parses one `FrameRecord` per line, checks
`schema_version == RECORD_SCHEMA_VERSION` per line (`UnsupportedVersion`
with the 1-based line number) and rejects malformed JSON (`Parse`).
Unknown fields are tolerated - older/newer minor shapes stay loadable;
new optional record fields (`score`, `tolerances`) default to absent.

## Reports

All reports implement `Display` for the `--analyze` CLI output and are pure
functions of `&Recording` - no engine state. `SimilarityReport` is the
exception: it takes `PatternSet` (a profile directory, not a recording).

| Report               | Phase | Question answered |
|----------------------|-------|-------------------|
| `UtilizationReport`  | 2.1   | How much of the configured tolerance does each distance index consume? p50/p95/p99/max per index, utilization bands, false negatives via labels |
| `DeviationReport`    | 2     | How far do the deviations sit from the fingerprint, per index? Two passes: `Candidates` folds recorded candidates, `Contacts` re-evaluates the best-fitting same-size contact subset of every frame (covers incumbent-tracked frames) |
| `ToleranceReport`    | 2.2   | Which `tolerances[]` does the data justify? `p99[i] × 1.25` per index, conservative scalar fallback, TOML snippet |
| `FlappyReport`       | 2.3   | How stable are contacts? Tracking-id lifetime histogram, dropout gaps (`frames_since_match` runs), partial-match share |
| `CollisionReport`    | 2.4   | Which foreign patterns explain labeled geometry? Per label × pattern: collided frames + required-tolerance percentiles |
| `SimilarityReport`   | —     | Which reduced contact sets of one figure satisfy another's fingerprint? `SimilarityPair::closest` sweeps subsets, `min_max_deviation` and `MatchRisk` classify the phantom-collision surface (profiles, not recordings) |
| `ScoringReport`      | 3.2   | Where does the acceptance threshold sit? τ-sweep (0.5–2.0) with recall/precision, own-vs-foreign score separation |
| `MlReport`           | 3.3   | Does a learned classifier beat the score rule? `LogisticBaseline` vs. `score ≤ 1.0` on a deterministic hold-out |
| `Dataset`            | 3.3   | JSONL export of labeled candidates (`features` = normalized deviations, `is_label` target) |
| `AnalysisReport`     | all   | Bundle of every recording-based report - the remote-interface counterpart of `--analyze` (`analyze_recording` on GraphQL/D-Bus/MCP); `AnalysisMode` selects the deviation pass |

`UtilizationReport` keys patterns by `PatternId` (stable identity); the
name is display-only. Label-only rows - a label that never produced a
candidate - appear with `pattern_id: None`.

## Types

Domain newtypes are defined in `model` and flow through the analysis
pipeline unchanged; the analysis crate adds only its own statistics types.

| Type               | Wraps            | Purpose |
|--------------------|------------------|---------|
| `Deviation`        | `Millimeters`    | Absolute gap `|measured − reference|` at one fingerprint index - a delta, not a distance |
| `Deviations`       | `Vec<Deviation>` | One frame's per-index deviation vector, parallel to `Distances` |
| `Distances`        | `Vec<Millimeters>` | Sorted pairwise-distance fingerprint of a point set |
| `MatchError`       | `Millimeters`    | `Deviations::total()` - candidate ranking (lower is better) |
| `MatchScore`       | `f64`            | `max_i dev[i]/tolerance[i]` - normalized; `≤ 1.0` accepts |
| `PatternTolerance` | `Millimeters`    | One tolerance slot |
| `PatternTolerances`| `Vec<PatternTolerance>` | Per-index tolerance vector, parallel to the fingerprint |
| `DeviationSamples` | `Vec<Deviation>` | One index's samples across frames - a column, not a `Deviations` vector |
| `RatioBands`       | 4×`FrameCount`   | `required/tolerance` quartile bins `<25/25-50/50-75/75-100 %`, named fields + `Display` |
| `Percentiles`      | `f64`×6          | min/p50/p95/p99/max/mean; `of(Iterator)` takes `Into<f64>` - the unit crosses only at the stats boundary |
| `Accuracy`         | `f64`            | Clamped `0.0..=1.0`; `Option<Accuracy>` = not evaluable |
| `DatasetRow`       | -                | Serialized per line: `frame` `is_label` `matched` `partial` `pattern_id` `score` `features[]` |

Borrowed-view types the analysis consumes directly from `model`:
`PatternSet` (pattern registry), `PointSet`/`PointBuf` (measured
positions), `Combination`/`CombinationBuf`/`Combinations` (index subsets
- `report.rs` enumerates same-size subsets per frame, `tangible.rs`
enumerates skip-pin sets), `TrackedSet`/`TrackedBuf` (alive tangibles).

## Design Rules

- **Shared schema**: records deserialize with the `model` types - no
  mirrored struct definitions.
- **Newtypes at boundaries**: accumulators count in `FrameCount`/
  `CandidateCount`, samples are `Deviation`/`MatchScore` values;
  `f64` is the working representation inside `Percentiles` only.
- **Deviation ≠ distance ≠ score**: three `Millimeters`-based meanings
  are three types; `score_of(&Deviations) -> MatchScore` is the only
  normalizing step.
- **Labels are ground truth**: utilization false negatives, the collision
  matrix, the score sweep and ML evaluation all require labeled frames;
  unlabeled recordings print a hint instead of empty tables.
- **No per-pin attribution**: `touch_indices` is an unordered combination -
  the flappy report measures per-contact stability, not per-pin.
