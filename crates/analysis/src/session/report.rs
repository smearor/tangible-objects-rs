use serde::Serialize;
use tangible_objects_model::AnalysisMode;
use tangible_objects_model::TangiblePattern;

use crate::CollisionReport;
use crate::DeviationReport;
use crate::FlappyReport;
use crate::MlReport;
use crate::Recording;
use crate::ScoringReport;
use crate::ToleranceReport;
use crate::UtilizationReport;

/// All recording-based reports bundled into one session analysis -
/// the remote-interface counterpart of `--analyze`.
#[derive(Debug, Serialize)]
pub struct AnalysisReport {
    /// Per-pattern uptime and false-negative statistics.
    pub utilization: UtilizationReport,
    /// Per-pin-pair deviation detail; `Candidates` folds the recorded
    /// generic candidates, `Contacts` re-evaluates the best-fitting
    /// same-size contact subset of every frame.
    pub deviation: DeviationReport,
    /// Suggested tolerances from measured deviations.
    pub tolerance: ToleranceReport,
    /// Contact-lifetime and dropout statistics.
    pub flappy: FlappyReport,
    /// Collision matrix across recorded labels and candidate patterns.
    pub collision: CollisionReport,
    /// Acceptance-threshold sweep for normalized match scores.
    pub scoring: ScoringReport,
    /// Logistic-baseline vs. rule-matcher accuracy comparison.
    pub ml: MlReport,
}

impl AnalysisReport {
    /// Runs every recording-based report; `mode` selects the deviation
    /// pass (`Contacts` needs `patterns` for the fingerprints).
    pub fn analyze(
        recording: &Recording,
        mode: AnalysisMode,
        patterns: &[TangiblePattern],
    ) -> Self {
        let deviation = match mode {
            AnalysisMode::Candidates => DeviationReport::analyze(recording),
            AnalysisMode::Contacts => DeviationReport::analyze_contacts(recording, patterns),
        };
        Self {
            utilization: UtilizationReport::analyze(recording),
            deviation,
            tolerance: ToleranceReport::analyze(recording),
            flappy: FlappyReport::analyze(recording),
            collision: CollisionReport::analyze(recording),
            scoring: ScoringReport::analyze(recording),
            ml: MlReport::analyze(recording),
        }
    }
}
