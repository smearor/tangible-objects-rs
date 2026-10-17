use serde::Serialize;

use tangible_objects_model::CandidateReport;
use tangible_objects_model::FrameNumber;
use tangible_objects_model::FrameRecord;
use tangible_objects_model::MatchScore;
use tangible_objects_model::PatternId;

/// One dataset row: a recorded candidate with its ground-truth target.
///
/// Serializes as one JSONL object per line.
#[derive(Debug, Serialize)]
pub struct DatasetRow {
    /// Frame the candidate was tested in.
    pub frame: FrameNumber,
    /// Whether the candidate's pattern is the frame's ground-truth label.
    pub is_label: bool,
    /// Whether the candidate matched under the recorded tolerances.
    pub matched: bool,
    /// Whether this was a partial (`num_points - 1`) candidate.
    pub partial: bool,
    /// Pattern the candidate was tested against.
    pub pattern_id: PatternId,
    /// Normalized deviation score (`max_i dev[i] / tolerance[i]`).
    pub score: MatchScore,
    /// Per-index normalized deviations `dev[i] / tolerance[i]` - the
    /// classifier features; `score` is their maximum.
    pub features: Vec<MatchScore>,
}

impl DatasetRow {
    /// Builds the dataset row of a candidate within a labeled frame.
    pub fn of(frame: &FrameRecord, label: &str, candidate: &CandidateReport) -> Self {
        Self {
            frame: frame.frame,
            is_label: candidate.matches_label(&label.into()),
            matched: candidate.matched,
            partial: candidate.partial,
            pattern_id: candidate.pattern_id,
            score: candidate.normalized_score(),
            features: candidate
                .deviations
                .iter()
                .enumerate()
                .map(|(i, dev)| MatchScore::new(dev.as_f64() / candidate.tolerance_at(i).as_f64()))
                .collect(),
        }
    }
}
