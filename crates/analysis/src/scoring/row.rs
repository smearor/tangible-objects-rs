use serde::Serialize;
use tangible_objects_model::FrameCount;
use tangible_objects_model::MatchScore;

/// One row of the acceptance-threshold sweep (TRAINING.md 3.2).
///
/// For threshold `tau`, a labeled frame counts as detected when the label's
/// best candidate has `normalized_score <= tau`.
#[derive(Debug, Serialize)]
pub struct ThresholdRow {
    /// Acceptance threshold on the normalized score (`1.0` = current binary
    /// matcher behaviour).
    pub tau: MatchScore,
    /// Labeled frames detected (true positives).
    pub true_positives: FrameCount,
    /// Labeled frames missed (false negatives).
    pub false_negatives: FrameCount,
    /// Labeled frames where a *different* pattern would have matched
    /// (false positives).
    pub false_positives: FrameCount,
}

impl ThresholdRow {
    /// Detected share of labeled frames.
    pub fn recall(&self) -> f64 {
        let total = self.true_positives + self.false_negatives;
        if total.as_u64() == 0 {
            0.0
        } else {
            self.true_positives.as_u64() as f64 / total.as_u64() as f64
        }
    }

    /// Share of correct predictions among positive ones.
    pub fn precision(&self) -> f64 {
        let total = self.true_positives + self.false_positives;
        if total.as_u64() == 0 {
            0.0
        } else {
            self.true_positives.as_u64() as f64 / total.as_u64() as f64
        }
    }
}
