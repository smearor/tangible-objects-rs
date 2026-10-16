# Profile Data Model

On-disk and in-memory representation of figure profiles — the module
`model/src/profile`. Profiles are hand-managed TOML files under `profiles/`;
they are the persistent form of a [`TangiblePattern`](../../docs/pattern/DATA_MODEL.md)
fingerprint.

```
crates/model/src/profile/
├── mod.rs      — module docs + re-exports
├── file.rs     → ProfileFile
├── dir.rs      → ProfileDir
└── error.rs    → ProfileError
```

## ProfileFile

The TOML-serializable figure profile — the on-disk representation of a
[`TangiblePattern`]. Contains the sorted pairwise-distance fingerprint plus
tolerance, name and optional reference points.

| Field          | Rust type           | TOML                  | Notes |
|----------------|---------------------|-----------------------|-------|
| `name`         | `String`            | `"TestFigur-5P"`      | Human-readable figure name |
| `num_points`   | `usize`             | `4`                   | Number of contact pins |
| `tolerance`    | `PatternTolerance`  | `3.0`                 | Transparent newtype over `Millimeters` — bare number in TOML; scalar fallback, always required |
| `tolerances`   | `Vec<PatternTolerance>` | `[2.2, 2.3, …]`   | Optional per-distance tolerances, parallel to `sorted_sides` — the `--analyze` tolerance suggestions render in exactly this format. `#[serde(default, skip_serializing_if = "Vec::is_empty")]` |
| `sorted_sides` | `Vec<Millimeters>`  | `[34.1, 36.8, …]`     | Sorted pairwise distances — the fingerprint |
| `points`       | `Vec<f64>`          | `[x0, y0, x1, y1, …]` | Optional reference positions, flattened; enables partial matching. `#[serde(default, skip_serializing_if = "Vec::is_empty")]` |

Example (`profiles/testfigur-5p.toml` shape):

```toml
name = "TestFigur-5P"
num_points = 4
tolerance = 3.0
# optional — per-index refinement (parallel to sorted_sides):
# tolerances = [3.00, 2.26, 2.28, 2.33, 2.35, 2.35]
sorted_sides = [33.32, 36.52, 35.11, 44.83, 53.5, 53.62]
points = [0.0, 0.0, 35.11, 0.0, 0.0, 33.32, 35.11, 33.32]
```

### Methods

| Method          | Signature                                              | Purpose |
|-----------------|--------------------------------------------------------|---------|
| `from_points`   | `(name, points, tolerance) -> ProfileFile`             | Computes the sorted pairwise distances from measured contact positions (used by `--train`) |
| `to_pattern`    | `(&self, id: PatternId) -> TangiblePattern`            | Converts into a matchable pattern; `ref_points` is `Some` only when `points.len() == num_points` |
| `save`          | `(&self, path) -> Result<(), ProfileError>`            | Pretty-prints TOML and writes the file |
| `load`          | `(path) -> Result<ProfileFile, ProfileError>`          | Reads and parses one TOML file |
| `load_dir`      | `(dir) -> ProfileDir`                                  | Loads every `*.toml` in a directory, sorted by filename |

## ProfileDir

Outcome of `ProfileFile::load_dir`: loaded profiles plus the files that
failed, so callers can warn instead of aborting. A missing directory yields an
empty result, not an error.

| Field      | Rust type                          | Notes |
|------------|------------------------------------|-------|
| `profiles` | `Vec<(PathBuf, ProfileFile)>`      | Loaded profiles with their source paths |
| `errors`   | `Vec<(PathBuf, ProfileError)>`     | Files that failed to load, with the error |

`into_patterns()` converts the loaded profiles into `TangiblePattern`s with
sequential `PatternId`s starting at 1 — deterministic because the files were
sorted by filename. **Note:** the pattern ID space is therefore
load-order-dependent; recordings reference `pattern_id` + `pattern_name`.

## ProfileError

Error enum (`thiserror`) for profile I/O:

| Variant       | Source              | Meaning |
|---------------|---------------------|---------|
| `Io`          | `std::io::Error`    | Reading or writing the file failed |
| `Deserialize` | `toml::de::Error`   | TOML did not parse into `ProfileFile` |
| `Serialize`   | `toml::ser::Error`  | `ProfileFile` could not be serialized |

## Design Rules

- `ProfileFile` is the **only** type in `model` performing file I/O; the rest
  of the crate is pure data.
- Tolerances and distances are `Millimeters` — profiles are device-independent
  because raw coordinates are calibrated before matching.
- `points` round-trips as a flat `f64` list for TOML readability; the
  `Vec<Vec2<Millimeters>>` reconstruction happens in `to_pattern`.
- Recordings (see [`../recording/DATA_MODEL.md`](../recording/DATA_MODEL.md))
  are generated artifacts and deliberately live outside `profiles/`.
