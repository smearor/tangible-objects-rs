use serde::Deserialize;
use serde::Serialize;

use super::MatchScore;
use crate::Millimeters;
use crate::PatternId;
use crate::PatternTolerance;
use crate::RecordLabel;

/// Diagnostic record of one tested point combination against a pattern.
///
/// Emitted by the engine's opt-in matching diagnostics for every candidate -
/// including rejected ones - so deviation distributions, per-pair tolerances
/// and collision risks can be analyzed offline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateReport {
    /// Pattern the combination was tested against.
    pub pattern_id: PatternId,
    /// Name of the figure profile the combination was tested against.
    pub pattern_name: String,
    /// Indices into the frame's `touches` forming this combination.
    pub touch_indices: Vec<usize>,
    /// Whether the combination matched the pattern within `tolerance`.
    pub matched: bool,
    /// Whether this was a partial attempt (`num_points - 1` contacts, one pin
    /// missing). Partial candidates compare against a subset fingerprint.
    pub partial: bool,
    /// Sorted pairwise distances measured on the combination's points.
    /// Index `i` corresponds to the pattern's `sorted_distances[i]` (or the
    /// subset fingerprint for partial candidates).
    pub measured_distances: Vec<Millimeters>,
    /// Absolute deviation `|measured[i] - reference[i]|` per distance index,
    /// parallel to `measured_distances`.
    pub deviations: Vec<Millimeters>,
    /// Scalar tolerance applied during the test (also the fallback for
    /// partial candidates, whose re-indexed fingerprint cannot use
    /// per-index tolerances).
    pub tolerance: PatternTolerance,
    /// Per-index tolerances applied during the test - only present when the
    /// pattern configured `tolerances`; absent means `tolerance` applied to
    /// every index. `#[serde(default)]` keeps older recordings loadable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tolerances: Vec<Millimeters>,
    /// Normalized deviation score (`max_i dev[i] / tolerance[i]`), recorded
    /// for matched and rejected candidates. `None` in older recordings - the
    /// analysis recomputes it from `deviations` and `tolerances`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<MatchScore>,
    /// Total absolute deviation across all distances (lower is better).
    /// Recorded for rejected candidates too - near-misses feed the
    /// collision analysis.
    pub match_error: Millimeters,
}

impl CandidateReport {
    /// The tolerance this candidate would need to match: the largest
    /// per-index deviation.
    ///
    /// Basis of the collision analysis - for a rejected candidate it is the
    /// tolerance at which it would have become a false positive. An empty
    /// deviation vector (degenerate candidate) needs zero tolerance.
    pub fn required_tolerance(&self) -> Millimeters {
        self.deviations
            .iter()
            .copied()
            .max_by(Millimeters::total_cmp)
            .unwrap_or(Millimeters::new(0.0))
    }

    /// Effective tolerance for distance index `i` - the recorded per-index
    /// value when present, else the scalar `tolerance`.
    pub fn tolerance_at(&self, index: usize) -> PatternTolerance {
        self.tolerances
            .get(index)
            .copied()
            .map(PatternTolerance::from)
            .unwrap_or(self.tolerance)
    }

    /// Normalized deviation score, recomputed from `deviations` and the
    /// effective tolerances. Falls back to the recorded `score` when
    /// `deviations` is empty; used by the offline threshold sweep so old and
    /// new recordings evaluate identically.
    pub fn normalized_score(&self) -> MatchScore {
        let computed = self
            .deviations
            .iter()
            .enumerate()
            .map(|(i, dev)| dev.as_f64() / self.tolerance_at(i).as_f64())
            .reduce(f64::max);
        match (self.score, computed) {
            (_, Some(score)) => MatchScore::new(score),
            (Some(recorded), None) => recorded,
            (None, None) => MatchScore::new(0.0),
        }
    }

    /// Whether a ground-truth label refers to this candidate's pattern.
    ///
    /// Labels carry either a profile name (`"TestFigur-4P"`) or a numeric
    /// pattern ID (`"4"`) - both forms are produced by `--record-label` and
    /// the interface APIs.
    pub fn matches_label(&self, label: &RecordLabel) -> bool {
        let label = label.as_str();
        self.pattern_name == label || self.pattern_id.0.to_string() == label
    }
}
