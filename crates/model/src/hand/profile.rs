//! `HandProfile` - a recorded five-contact handprint (SPELLS.md):
//! the constellation a whole-hand tap lands with, registered so a
//! spell's `five-finger-tap` arming can require *this* hand
//! (per-user spells).

use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;

use crate::Distances;
use crate::PointSet;

use super::HandName;

/// A named hand constellation - serialized from
/// `hands.d/*.toml` like [`crate::Spell`]:
///
/// ```toml
/// name = "alice"
/// tolerance = 8.0
/// points = [[100.0, 300.0], [160.0, 280.0], [210.0, 320.0], [255.0, 340.0], [300.0, 310.0]]
/// ```
///
/// Matching reuses the profile fingerprint: the sorted pairwise
/// distances of the observed tap must deviate at most `tolerance`
/// per index - rotation- and translation-invariant like every
/// `sorted_sides` comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandProfile {
    /// Identity - addresses the handprint for unregister and the
    /// store, and names the binding `five-finger-tap` arming
    /// requires.
    pub name: HandName,
    /// Matching tolerance per pairwise distance in millimeters.
    pub tolerance: Millimeters,
    /// Sorted pairwise distances of `points` - the fingerprint a
    /// tap's measured fingerprint is compared index-by-index
    /// against (same metric as `ProfileFile::sorted_sides`).
    pub sorted_sides: Distances,
    /// The recorded contact constellation in surface millimeters.
    #[serde(with = "smearor_commons_geometry::serde_points")]
    pub points: Vec<Vec2<Millimeters>>,
}

impl HandProfile {
    /// Creates a handprint from measured contact positions -
    /// computes the sorted pairwise-distance fingerprint like
    /// [`crate::ProfileFile::from_points`].
    pub fn from_points(
        name: impl Into<HandName>,
        points: &[Vec2<Millimeters>],
        tolerance: Millimeters,
    ) -> Self {
        Self {
            name: name.into(),
            tolerance,
            sorted_sides: PointSet::new(points).distances(),
            points: points.to_vec(),
        }
    }

    /// Whether `points` is this hand's constellation: same point
    /// count and every pairwise distance within `tolerance` of the
    /// fingerprint (per-index, like the pattern matcher).
    pub fn matches(&self, points: &[Vec2<Millimeters>]) -> bool {
        PointSet::new(points)
            .distances()
            .max_deviation(&self.sorted_sides)
            .as_millimeters()
            <= self.tolerance
    }

    /// Loads a single handprint `.toml` file.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }

    /// Loads all `.toml` handprint files from a directory, sorted
    /// by filename. Unreadable or invalid files are skipped and
    /// returned in the error list.
    pub fn load_dir(dir: &Path) -> (Vec<Self>, Vec<(PathBuf, std::io::Error)>) {
        let mut hands = Vec::new();
        let mut errors = Vec::new();
        let Ok(entries) = std::fs::read_dir(dir) else {
            return (hands, errors);
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        files.sort();
        for file in files {
            match Self::load(&file) {
                Ok(hand) => hands.push(hand),
                Err(error) => errors.push((file, error)),
            }
        }
        (hands, errors)
    }

    /// Saves the handprint as a TOML file (runtime-store
    /// persistence, `hands.d`).
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        std::fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::HandProfile;
    use smearor_commons_geometry::Millimeters;
    use smearor_commons_geometry::Vec2;

    fn hand_points() -> Vec<Vec2<Millimeters>> {
        // A splayed five-finger constellation (~real hand span).
        vec![
            Vec2::new(0.0, 40.0),
            Vec2::new(35.0, 15.0),
            Vec2::new(65.0, 0.0),
            Vec2::new(95.0, 20.0),
            Vec2::new(110.0, 55.0),
        ]
    }

    #[test]
    fn same_constellation_matches_shifted() {
        let hand = HandProfile::from_points("alice", &hand_points(), Millimeters::new(8.0));
        // Translated + slightly jittered - still Alice's hand.
        let shifted: Vec<Vec2<Millimeters>> = hand_points()
            .iter()
            .map(|point| *point + Vec2::new(200.0, 100.0))
            .collect();
        assert!(hand.matches(&shifted));
    }

    #[test]
    fn different_constellation_rejects() {
        let hand = HandProfile::from_points("alice", &hand_points(), Millimeters::new(8.0));
        // A cramped five-contact cluster is a different hand.
        let tiny: Vec<Vec2<Millimeters>> = hand_points()
            .iter()
            .map(|point| Vec2::new(point.x * 0.4, point.y * 0.4))
            .collect();
        assert!(!hand.matches(&tiny));
    }

    #[test]
    fn wrong_point_count_rejects() {
        let hand = HandProfile::from_points("alice", &hand_points(), Millimeters::new(8.0));
        assert!(!hand.matches(&hand_points()[..4]));
    }

    #[test]
    fn toml_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let hand = HandProfile::from_points("alice", &hand_points(), Millimeters::new(8.0));
        let serialized = toml::to_string_pretty(&hand)?;
        let loaded: HandProfile = toml::from_str(&serialized)?;
        assert_eq!(loaded.name.as_str(), "alice");
        assert_eq!(loaded.points.len(), 5);
        assert_eq!(loaded.sorted_sides, hand.sorted_sides);
        Ok(())
    }
}
