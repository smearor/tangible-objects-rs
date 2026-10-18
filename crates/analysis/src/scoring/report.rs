use std::fmt::Display;

use serde::Serialize;
use tangible_objects_model::FrameCount;
use tangible_objects_model::MatchScore;

use super::ThresholdRow;
use crate::Percentiles;
use crate::Recording;

/// Sweep points of the acceptance threshold (τ on the normalized score).
const THRESHOLDS: &[f64] = &[0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];

/// Evaluation of score-based acceptance vs. the binary matcher
/// (TRAINING.md 3.2).
///
/// For every labeled frame the best candidate per pattern is scored with
/// [`CandidateReport::normalized_score`] - the sweep shows recall/precision
/// at each acceptance threshold τ, locating the trade-off the binary
/// matcher (τ = 1.0) sits on.
///
/// [`CandidateReport::normalized_score`]: tangible_objects_model::CandidateReport::normalized_score
#[derive(Debug, Serialize)]
pub struct ScoringReport {
    /// Labeled frames in the recording (0 → sweep skipped).
    pub labeled_frames: FrameCount,
    /// Per-threshold detection statistics.
    pub rows: Vec<ThresholdRow>,
    /// Scores of the best candidate per labeled frame - the label's own
    /// pattern (detected) and foreign patterns (collision pressure).
    pub own_score: Option<Percentiles>,
    /// Scores of the best foreign candidate per labeled frame.
    pub foreign_score: Option<Percentiles>,
}

impl ScoringReport {
    /// Evaluates score-based matching on a loaded recording.
    ///
    /// Needs ground-truth labels; on unlabeled recordings the report is
    /// empty and the display notes that labels are required.
    pub fn analyze(recording: &Recording) -> Self {
        let mut rows: Vec<ThresholdRow> = THRESHOLDS
            .iter()
            .map(|tau| ThresholdRow {
                tau: MatchScore::new(*tau),
                true_positives: FrameCount::new(0),
                false_negatives: FrameCount::new(0),
                false_positives: FrameCount::new(0),
            })
            .collect();
        let mut own_scores: Vec<MatchScore> = Vec::new();
        let mut foreign_scores: Vec<MatchScore> = Vec::new();
        let mut labeled_frames = FrameCount::new(0);

        for frame in recording.frames() {
            let Some(label) = &frame.label else { continue };
            labeled_frames += 1;

            let mut own_best: Option<MatchScore> = None;
            let mut foreign_best: Option<MatchScore> = None;
            for candidate in &frame.candidates {
                let score = candidate.normalized_score();
                let better = |best: Option<MatchScore>| match best {
                    Some(current) if current <= score => Some(current),
                    _ => Some(score),
                };
                if candidate.matches_label(label) {
                    own_best = better(own_best);
                } else {
                    foreign_best = better(foreign_best);
                }
            }
            if let Some(score) = own_best {
                own_scores.push(score);
            }
            if let Some(score) = foreign_best {
                foreign_scores.push(score);
            }

            for row in &mut rows {
                match own_best {
                    Some(score) if score <= row.tau => row.true_positives += 1,
                    _ => row.false_negatives += 1,
                }
                if let Some(score) = foreign_best
                    && score <= row.tau
                {
                    row.false_positives += 1;
                }
            }
        }

        Self {
            labeled_frames,
            rows,
            own_score: Percentiles::of(own_scores.iter().copied()),
            foreign_score: Percentiles::of(foreign_scores.iter().copied()),
        }
    }
}

impl Display for ScoringReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "== Score sweep == ({} labeled frames)",
            self.labeled_frames
        )?;
        if self.labeled_frames.as_u64() == 0 {
            writeln!(f, "  (no labeled frames - record with --record-label)")?;
            return Ok(());
        }
        if let Some(own) = self.own_score {
            writeln!(
                f,
                "  own pattern scores:   p50={:.2} p95={:.2} p99={:.2}",
                own.p50, own.p95, own.p99
            )?;
        }
        if let Some(foreign) = self.foreign_score {
            writeln!(
                f,
                "  foreign best scores:  min={:.2} p50={:.2} p95={:.2}",
                foreign.min, foreign.p50, foreign.p95
            )?;
        }
        writeln!(f, "  τ      recall  precision  (tp/fn/fp)")?;
        for row in &self.rows {
            writeln!(
                f,
                "  {:.2}   {:>5.1}%  {:>6.1}%   ({}/{}/{})",
                row.tau.as_f64(),
                row.recall() * 100.0,
                row.precision() * 100.0,
                row.true_positives,
                row.false_negatives,
                row.false_positives
            )?;
        }
        Ok(())
    }
}
