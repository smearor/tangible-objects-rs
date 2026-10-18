use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;

use crate::Millimeters;
use crate::PointSet;

/// Sorted pairwise-distance fingerprint of a point set.
///
/// Invariant: elements are sorted ascending - matching compares
/// fingerprints index-by-index against `tolerance_at(i)`. A *deviation*
/// vector is a different thing (element-wise error, not sorted); it
/// stays `Vec<Millimeters>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Distances(pub Vec<Millimeters>);

impl Distances {
    /// Sorted pairwise distances of a point set - the fingerprint used
    /// for matching.
    pub fn of(points: PointSet<'_>) -> Self {
        let mut distances = Vec::new();
        for (index, a) in points.iter().enumerate() {
            for b in &points[index + 1..] {
                distances.push(a.distance(b));
            }
        }
        distances.sort_by(Millimeters::total_cmp);
        Self(distances)
    }

    /// Largest absolute per-index deviation against another fingerprint.
    /// Returns [`f64::MAX`] millimeters for empty or length-mismatched
    /// fingerprints - a maximal, never-matching collision distance.
    pub fn max_deviation(&self, other: &Self) -> Millimeters {
        if self.len() != other.len() || self.is_empty() {
            return Millimeters::new(f64::MAX);
        }
        self.iter()
            .zip(other.iter())
            .map(|(a, b)| (*a - *b).abs())
            .max_by(Millimeters::total_cmp)
            .unwrap_or(Millimeters::new(f64::MAX))
    }

    /// Absolute per-index deviations against another fingerprint;
    /// empty when the lengths differ.
    pub fn deviations(&self, other: &Self) -> Vec<Millimeters> {
        if self.len() != other.len() {
            return Vec::new();
        }
        self.iter()
            .zip(other.iter())
            .map(|(a, b)| (*a - *b).abs())
            .collect()
    }
}

impl From<Vec<Millimeters>> for Distances {
    fn from(distances: Vec<Millimeters>) -> Self {
        Self(distances)
    }
}

impl From<Distances> for Vec<Millimeters> {
    fn from(distances: Distances) -> Self {
        distances.0
    }
}

impl Deref for Distances {
    type Target = [Millimeters];

    /// Distance lists read like a slice.
    fn deref(&self) -> &[Millimeters] {
        &self.0
    }
}
