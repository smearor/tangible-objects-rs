use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

use crate::Millimeters;

/// Normalized deviation score of a candidate: `max_i dev[i] / tolerance[i]`.
///
/// Dimensionless - `score <= 1.0` means the candidate matched; values above
/// quantify how far a near-miss was off. Used for score-based ranking and
/// offline threshold sweeps.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MatchScore(pub f64);

impl MatchScore {
    /// Creates a match score.
    pub const fn new(score: f64) -> Self {
        Self(score)
    }

    /// `error / tolerance` - the normalized-deviation construction:
    /// `<= 1.0` means the candidate matched. Shared by `score_of` and
    /// the partial-match score paths.
    pub fn normalized(error: Millimeters, tolerance: Millimeters) -> Self {
        Self(error.as_f64() / tolerance.as_f64())
    }

    /// The higher (worse) of two scores - the aggregation of
    /// `Deviations::max_score`. Not `Ord::max`: `f64` values admit
    /// NaN, so `MatchScore` can carry only `PartialOrd`.
    pub fn max(self, other: Self) -> Self {
        if other > self { other } else { self }
    }

    /// The raw score.
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}

impl From<f64> for MatchScore {
    fn from(score: f64) -> Self {
        Self(score)
    }
}

impl Default for MatchScore {
    /// The neutral score of `max` aggregation - the identity of
    /// `max` on the non-negative score domain (`0.0` = perfect).
    fn default() -> Self {
        Self(0.0)
    }
}

impl From<MatchScore> for f64 {
    fn from(score: MatchScore) -> Self {
        score.0
    }
}

impl Display for MatchScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}
