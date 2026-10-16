use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;

/// Normalized deviation score of a candidate: `max_i dev[i] / tolerance[i]`.
///
/// Dimensionless — `score <= 1.0` means the candidate matched; values above
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
