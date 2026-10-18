use serde::Deserialize;
use serde::Serialize;

use crate::Millimeters;

/// Total absolute deviation of a candidate - the sum over the
/// [`crate::Deviations`] vector; lower is better.
///
/// Distinct from [`crate::MatchScore`]: the error is an absolute
/// millimeter sum used for ranking same-scale candidates and recorded
/// for collision analysis, while the score is the tolerance-normalized
/// acceptance metric.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MatchError(pub Millimeters);

impl MatchError {
    /// Wraps a millimeter error value.
    pub const fn new(error: Millimeters) -> Self {
        Self(error)
    }

    /// The wrapped millimeters.
    pub fn as_millimeters(self) -> Millimeters {
        self.0
    }

    /// The error as a plain number.
    pub fn as_f64(self) -> f64 {
        self.0.as_f64()
    }

    /// Total-order comparison for ranking (f64 has no `Ord`).
    pub fn total_cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl From<Millimeters> for MatchError {
    /// A millimeter sum IS a match error.
    fn from(error: Millimeters) -> Self {
        Self(error)
    }
}
