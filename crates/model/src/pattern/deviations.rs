use std::ops::Deref;

use serde::Deserialize;
use serde::Serialize;

use crate::Deviation;
use crate::MatchError;
use crate::Millimeters;

/// Per-index absolute deviations `|measured[i] - reference[i]|`
/// between a measured fingerprint and a reference fingerprint -
/// parallel to [`crate::Distances`], not a fingerprint itself.
///
/// A deviations vector has no ordering invariant (its order is the
/// fingerprint's distance index order) and its values are deltas, not
/// distances - the distinction a `Distances`/`Deviations` split makes
/// visible.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Deviations(pub Vec<Deviation>);

impl Deviations {
    /// The deviation slice.
    pub fn as_slice(&self) -> &[Deviation] {
        &self.0
    }

    /// Total absolute deviation across all indices - the candidate's
    /// match error (lower is better).
    pub fn total(&self) -> MatchError {
        crate::MatchError::new(Millimeters::new(
            self.0.iter().map(|deviation| deviation.as_f64()).sum(),
        ))
    }
}

impl From<Vec<Deviation>> for Deviations {
    /// A deviation vector IS a deviations set.
    fn from(deviations: Vec<Deviation>) -> Self {
        Self(deviations)
    }
}

impl Deref for Deviations {
    type Target = [Deviation];

    /// Deviations view their slice transparently.
    fn deref(&self) -> &[Deviation] {
        &self.0
    }
}
