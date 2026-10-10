use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use super::ProfileDir;
use super::ProfileError;
use crate::Millimeters;
use crate::PatternId;
use crate::PatternTolerance;
use crate::TangiblePattern;
use crate::Vec2;

/// TOML-serializable figure profile.
///
/// This is the on-disk representation of a [`TangiblePattern`]: the sorted pairwise
/// distance fingerprint plus tolerance, name and optional reference points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileFile {
    /// Human-readable figure name.
    pub name: String,
    /// Number of contact pins.
    pub num_points: usize,
    /// Matching tolerance in mm.
    pub tolerance: PatternTolerance,
    /// Sorted pairwise distances of the reference geometry (the fingerprint).
    pub sorted_sides: Vec<f64>,
    /// Original contact positions in mm, flattened as `[x0, y0, x1, y1, ...]`.
    /// Enables partial matching when one contact pin is missing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<f64>,
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
        distances.sort_by(f64::total_cmp);
        let pts: Vec<f64> = points.iter().flat_map(|p| [p.x, p.y]).collect();
        Self {
            name: name.to_string(),
            num_points: points.len(),
            tolerance,
            sorted_sides: distances,
            points: pts,
        }
    }

    /// Converts this profile into a [`TangiblePattern`] ready for matching.
    pub fn to_pattern(&self, id: PatternId) -> TangiblePattern {
        let ref_points: Vec<Vec2<Millimeters>> = self
            .points
            .chunks(2)
            .filter(|c| c.len() == 2)
            .map(|c| Vec2::new(c[0], c[1]))
            .collect();
        TangiblePattern {
            id,
            name: self.name.clone(),
            num_points: self.num_points,
            sorted_distances: self.sorted_sides.clone(),
            tolerance: self.tolerance,
            ref_points: if ref_points.len() == self.num_points {
                Some(ref_points)
            } else {
                None
            },
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

        assert_eq!(loaded.name, "triangle");
        assert_eq!(loaded.num_points, 3);
        assert_eq!(loaded.tolerance.as_f64(), 5.0);
        assert_eq!(loaded.sorted_sides, profile.sorted_sides);
        assert_eq!(loaded.points.len(), 6);
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
        assert_eq!(pattern.name, "triangle");
        assert_eq!(pattern.num_points, 3);
        assert_eq!(pattern.ref_points, Some(points));
    }

    #[test]
    fn load_dir_on_missing_directory_returns_empty() {
        let dir = ProfileFile::load_dir(&PathBuf::from("/nonexistent/profiles"));
        assert!(dir.profiles.is_empty());
        assert!(dir.errors.is_empty());
    }
}
