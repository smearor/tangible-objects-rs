# Pattern Data Model

Geometric figure patterns and their matching within a frame - the module
`model/src/pattern`. A figure is recognized by an invariant fingerprint: all
pairwise distances between its contact pins, sorted ascending. Matching is
therefore position- and rotation-invariant.

```
crates/model/src/pattern/
├── mod.rs        - module docs + re-exports
├── id.rs         → PatternId
├── tangible.rs   → TangiblePattern
├── match.rs      → PatternMatch
└── tolerance.rs  → PatternTolerance
```

## TangiblePattern

Runtime pattern loaded from a [`ProfileFile`](../profile/DATA_MODEL.md). For
n pins the fingerprint consists of all `n·(n−1)/2` pairwise distances, sorted
ascending.

| Field              | Rust type                          | Notes |
|--------------------|------------------------------------|-------|
| `id`               | `PatternId`                        | Assigned sequentially at profile load (filename order) |
| `name`             | `String`                           | Figure name from the profile |
| `num_points`       | `usize`                            | Required pin count |
| `sorted_distances` | `Vec<Millimeters>`                 | The fingerprint |
| `tolerance`        | `PatternTolerance`                 | Scalar max deviation per distance - fallback for every index and for partial matching |
| `tolerances`       | `Vec<PatternTolerance>`            | Optional per-distance tolerances parallel to `sorted_distances`; empty or length-mismatched → scalar fallback (TRAINING.md 3.1) |
| `ref_points`       | `Option<Vec<Vec2<Millimeters>>>`   | Original pin positions; required for partial matching |

### Matching methods

| Method            | Signature                                              | Semantics |
|-------------------|--------------------------------------------------------|-----------|
| `from_points`     | `(id, name, points, tolerance) -> TangiblePattern`     | Builds the fingerprint from reference positions; stores them as `ref_points` |
| `tolerance_at`    | `(&self, index) -> PatternTolerance`                   | Effective tolerance for distance index `i` - per-index value or scalar fallback |
| `matches`         | `(&self, points) -> bool`                              | Exact match: `points.len() == num_points` and every deviation ≤ `tolerance_at(i)` |
| `match_score`     | `(&self, points) -> f64`                               | Normalized deviation score `max_i dev[i]/tol[i]`; `≤ 1.0` ⟺ `matches` - the score-based ranking metric |
| `score_of`        | `(&self, deviations) -> f64`                           | Score of a precomputed deviation vector |
| `match_error`     | `(&self, points) -> Millimeters`                       | Total absolute deviation across all distances (lower is better) |
| `matches_partial` | `(&self, points) -> Option<(Vec<usize>, Millimeters)>` | Tests `num_points - 1` contacts against every one-pin-dropped subset; tolerance raised ×1.5; returns matched reference indices + error |

`sorted_pairwise_distances` (private) computes the fingerprint; comparison
zips `measured[i]` against `sorted_distances[i]` - the same index alignment
that `CandidateReport.deviations` records in
[recordings](../recording/DATA_MODEL.md).

## PatternMatch

A successful match within a single input frame - produced by the engine's
`find_matches`, consumed by the `Tracker`.

| Field           | Rust type             | Notes |
|-----------------|-----------------------|-------|
| `pattern_id`    | `PatternId`           | Matched pattern |
| `pattern_name`  | `String`              | `"(partial)"` suffix for partial matches |
| `center`        | `Vec2<Millimeters>`   | Centroid of the matched touch points |
| `rotation_rad`  | `RotationRad`         | Estimated orientation |
| `touch_indices` | `Vec<usize>`          | Indices into the frame's touch list |

`PatternMatch::rotation_of(points, center)` estimates orientation: for 3
points the vertex opposite the shortest side defines it; for 4+ points the
point farthest from the centroid. Returns `0.0` for empty sets.

## Newtypes

| Type               | Wraps          | Purpose |
|--------------------|----------------|---------|
| `PatternId`        | `u32`          | Runtime ID; transparent serde, `Display`, GraphQL scalar, zvariant `Type` |
| `PatternTolerance` | `Millimeters`  | Tolerance so it cannot be confused with a distance; serializes as bare number for profile compatibility |

## Design Rules

- Fingerprint comparison is index-aligned: `measured[i]` vs.
  `sorted_distances[i]` - per-index `tolerances[i]` keeps this indexing.
  Partial matching stays scalar by design: dropping a pin re-indexes the
  fingerprint, so per-index tolerances cannot apply there.
- `matches`/`match_error`/`matches_partial` are pure functions of the point
  set - no engine state, freely callable for diagnostics (`CandidateReport`).
- `PatternMatch` derives `Debug, Clone` only - it is engine output; the
  recording pipeline wraps it, it is not serialized directly.
