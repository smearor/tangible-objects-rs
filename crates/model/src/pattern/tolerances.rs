use std::ops::Deref;

use serde::Deserialize;
use serde::Serialize;

use super::PatternTolerance;
use crate::Millimeters;

/// Per-index tolerances of a pattern - parallel to the sorted
/// distance fingerprint.
///
/// A set of `n` pins has `n(n-1)/2` distances and as many tolerance
/// slots; `tolerances[i]` refines the scalar [`PatternTolerance`]
/// fallback for distance `i`. An empty vector means "not configured"
/// - the scalar applies to every index.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PatternTolerances(pub Vec<PatternTolerance>);

impl PatternTolerances {
    /// Whether no per-index tolerances are configured.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The tolerances as plain millimeter values.
    pub fn as_millimeters(&self) -> Vec<Millimeters> {
        self.0
            .iter()
            .map(|tolerance| tolerance.as_millimeters())
            .collect()
    }
}

impl From<Vec<PatternTolerance>> for PatternTolerances {
    /// A tolerance vector IS a pattern-tolerance set.
    fn from(tolerances: Vec<PatternTolerance>) -> Self {
        Self(tolerances)
    }
}

impl Deref for PatternTolerances {
    type Target = [PatternTolerance];

    /// Tolerance sets view their slice transparently.
    fn deref(&self) -> &[PatternTolerance] {
        &self.0
    }
}
