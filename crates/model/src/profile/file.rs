use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use super::ProfileDir;
use super::ProfileError;
use crate::Distances;
use crate::FrameCount;
use crate::Millimeters;
use crate::PatternId;
use crate::PatternName;
use crate::PatternTolerance;
use crate::PatternTolerances;
use crate::PinCount;
use crate::Region;
use crate::TangiblePattern;
use crate::Vec2;

/// TOML-serializable figure profile.
///
/// This is the on-disk representation of a [`TangiblePattern`]: the sorted pairwise
/// distance fingerprint plus tolerance, name and optional reference points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileFile {
    /// Human-readable figure name.
    pub name: PatternName,
    /// Whether the profile is enabled - a disabled profile is skipped at
    /// startup (`enabled = false` keeps the file without registering).
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Whether a partial match may *spawn* this tangible
    /// (SPAWN_AS_PARTIAL.md). `false` (default, conservative): only
    /// exact matches create a track - reduced contact sets of other
    /// figures can then never spawn this pattern as a phantom. Partials
    /// still sustain an existing incumbent regardless of this flag.
    #[serde(default)]
    pub spawn_on_partial: bool,
    /// Frames a partial spawn must hold before the tangible leaves its
    /// pending state and emits `TangibleAdded` - phantom partials die
    /// inside this window. Only meaningful with `spawn_on_partial`.
    #[serde(default = "default_spawn_stability")]
    pub spawn_stability: FrameCount,
    /// Number of contact pins.
    pub num_points: usize,
    /// Matching tolerance for the distance fingerprint.
    ///
    /// Scalar fallback - always required; per-index `tolerances` below refine
    /// it where configured.
    pub tolerance: PatternTolerance,
    /// Optional per-distance tolerances, parallel to `sorted_sides`.
    /// Omitted when empty; the analysis `--analyze` tolerance suggestions
    /// render in exactly this format (`tolerances = [...]`).
    #[serde(default, skip_serializing_if = "PatternTolerances::is_empty")]
    pub tolerances: PatternTolerances,
    /// Sorted pairwise distances of the reference geometry (the fingerprint).
    pub sorted_sides: Distances,
    /// Original contact positions - enables partial matching when one
    /// contact pin is missing. Serialized flat as `[x0, y0, x1, y1, ...]`
    /// for on-disk compatibility.
    #[serde(default, skip_serializing_if = "Vec::is_empty", with = "flat_points")]
    pub points: Vec<Vec2<Millimeters>>,
    /// Detection regions in millimeters restricting where this figure may
    /// match (PROFILE_REGIONS.md). Empty = anywhere on the surface.
    /// Serialized as TOML `[[regions]]` blocks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<Region>,
}

/// Default enabled state for profile files (`enabled = true`).
fn default_enabled() -> bool {
    true
}

/// Default partial-spawn confirmation window (`spawn_stability = 15`
/// frames, ~0.25 s at 60 Hz). Phantom partials die inside this window
/// before they can emit `TangibleAdded`.
pub const DEFAULT_SPAWN_STABILITY: FrameCount = FrameCount::new(15);
fn default_spawn_stability() -> FrameCount {
    DEFAULT_SPAWN_STABILITY
}

impl ProfileFile {
    /// Creates a profile from measured contact positions.
    ///
    /// Computes all pairwise distances and sorts them ascending.
    pub fn from_points(
        name: &str,
        points: &[Vec2<Millimeters>],
        tolerance: impl Into<PatternTolerance>,
    ) -> Self {
        let tolerance = tolerance.into();
        let mut distances = Vec::new();
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                distances.push(points[i].distance(&points[j]));
            }
        }
        distances.sort_by(Millimeters::total_cmp);
        Self {
            name: name.into(),
            enabled: true,
            spawn_on_partial: false,
            spawn_stability: default_spawn_stability(),
            num_points: points.len(),
            tolerance,
            tolerances: PatternTolerances::default(),
            sorted_sides: Distances(distances),
            points: points.to_vec(),
            regions: Vec::new(),
        }
    }

    /// Converts this profile into a [`TangiblePattern`] ready for matching.
    pub fn to_pattern(&self, id: PatternId) -> TangiblePattern {
        TangiblePattern {
            id,
            name: self.name.clone(),
            spawn_on_partial: self.spawn_on_partial,
            spawn_stability: self.spawn_stability,
            num_points: PinCount::new(self.num_points),
            sorted_distances: self.sorted_sides.clone(),
            tolerance: self.tolerance,
            tolerances: self.tolerances.clone(),
            ref_points: if self.points.len() == self.num_points {
                Some(self.points.clone())
            } else {
                None
            },
            regions: self.regions.clone(),
        }
    }

    /// Saves the profile to a TOML file.
    pub fn save(&self, path: &Path) -> Result<(), ProfileError> {
        let toml_str = toml::to_string_pretty(self)?;
        fs::write(path, toml_str)?;
        Ok(())
    }

    /// Loads a profile from a TOML file.
    pub fn load(path: &Path) -> Result<Self, ProfileError> {
        let content = fs::read_to_string(path)?;
        let profile = toml::from_str(&content)?;
        Ok(profile)
    }

    /// Loads all `.toml` profile files from a directory, sorted by filename.
    ///
    /// Individual files that fail to load are skipped; their paths and errors
    /// are returned in [`ProfileDir::errors`]. Missing directories yield an
    /// empty result rather than an error.
    pub fn load_dir(dir: &Path) -> ProfileDir {
        let mut result = ProfileDir::default();

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return result,
        };

        let mut files: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|p| p.extension().map(|ext| ext == "toml").unwrap_or(false))
            .collect();
        files.sort();

        for file in files {
            match ProfileFile::load(&file) {
                Ok(profile) => result.profiles.push((file, profile)),
                Err(error) => result.errors.push((file, error)),
            }
        }

        result
    }
}

/// Serde adapter keeping `points` as a flat `[x0, y0, ...]` TOML array
/// while the field itself is `Vec<Vec2<Millimeters>>` (on-disk format
/// compatibility).
mod flat_points {
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serializer;
    use serde::ser::SerializeSeq;

    use crate::Millimeters;
    use crate::Vec2;

    pub fn serialize<S>(points: &[Vec2<Millimeters>], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(points.len() * 2))?;
        for point in points {
            seq.serialize_element(&point.x)?;
            seq.serialize_element(&point.y)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec2<Millimeters>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let flat = Vec::<f64>::deserialize(deserializer)?;
        Ok(flat
            .chunks(2)
            .filter(|chunk| chunk.len() == 2)
            .map(|chunk| Vec2::new(chunk[0], chunk[1]))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::ProfileFile;
    use crate::PatternId;
    use crate::Vec2;

    #[test]
    fn toml_round_trip_preserves_fields() {
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let profile = ProfileFile::from_points("triangle", &points, 5.0);

        let serialized = match toml::to_string(&profile) {
            Ok(s) => s,
            Err(_) => panic!("serialization failed"),
        };
        let loaded: ProfileFile = match toml::from_str(&serialized) {
            Ok(p) => p,
            Err(_) => panic!("deserialization failed"),
        };

        assert_eq!(loaded.name.as_str(), "triangle");
        assert_eq!(loaded.num_points, 3);
        assert_eq!(loaded.tolerance.as_f64(), 5.0);
        assert_eq!(loaded.sorted_sides, profile.sorted_sides);
        assert_eq!(loaded.points.len(), 3);
        // Empty tolerances stay absent - backward-compatible TOML.
        assert!(!serialized.contains("tolerances"));
        assert!(loaded.tolerances.is_empty());
    }

    #[test]
    fn tolerances_round_trip_and_reach_the_pattern() {
        let toml = r#"
            name = "square"
            num_points = 4
            tolerance = 3.0
            tolerances = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
            sorted_sides = [40.0, 40.0, 40.0, 40.0, 56.57, 56.57]
        "#;
        let profile: ProfileFile = match toml::from_str(toml) {
            Ok(p) => p,
            Err(_) => panic!("deserialization failed"),
        };
        assert_eq!(profile.tolerances.len(), 6);
        let pattern = profile.to_pattern(PatternId::new(1));
        assert_eq!(pattern.tolerances.len(), 6);
        assert_eq!(pattern.tolerance_at(0).as_f64(), 1.0);
        assert_eq!(pattern.tolerance_at(6).as_f64(), 3.0); // scalar fallback
    }

    #[test]
    fn to_pattern_restores_ref_points() {
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let profile = ProfileFile::from_points("triangle", &points, 5.0);
        let pattern = profile.to_pattern(PatternId::new(7));

        assert_eq!(pattern.id, PatternId::new(7));
        assert_eq!(pattern.name.as_str(), "triangle");
        assert_eq!(pattern.num_points, 3);
        assert_eq!(pattern.ref_points, Some(points));
    }

    #[test]
    fn regions_round_trip_and_reach_the_pattern() {
        let toml = r#"
            name = "left-pawn"
            num_points = 3
            tolerance = 3.0
            sorted_sides = [40.0, 40.0, 56.57]

            [[regions]]
            x_min = 0.0
            y_min = 0.0
            x_max = 400.0
            y_max = 800.0
        "#;
        let profile: ProfileFile = match toml::from_str(toml) {
            Ok(p) => p,
            Err(_) => panic!("deserialization failed"),
        };
        assert_eq!(profile.regions.len(), 1);
        let pattern = profile.to_pattern(PatternId::new(1));
        assert_eq!(pattern.regions.len(), 1);
        assert!(
            pattern.accepts(
                (&[
                    Vec2::new(10.0, 10.0),
                    Vec2::new(50.0, 50.0),
                    Vec2::new(100.0, 100.0),
                ])
                    .into()
            )
        );
        assert!(
            !pattern.accepts(
                (&[
                    Vec2::new(10.0, 10.0),
                    Vec2::new(50.0, 50.0),
                    Vec2::new(500.0, 100.0),
                ])
                    .into()
            )
        );
    }

    #[test]
    fn load_dir_on_missing_directory_returns_empty() {
        let dir = ProfileFile::load_dir(&PathBuf::from("/nonexistent/profiles"));
        assert!(dir.profiles.is_empty());
        assert!(dir.errors.is_empty());
    }
}
