use serde::Serialize;

use crate::Millimeters;
use crate::PatternId;
use crate::PatternTolerance;

/// Diagnostic record of one tested point combination against a pattern.
///
/// Emitted by the engine's opt-in matching diagnostics for every candidate —
/// including rejected ones — so deviation distributions, per-pair tolerances
/// and collision risks can be analyzed offline.
#[derive(Debug, Clone, PartialEq, Serialize)]
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
    /// Tolerance applied during the test.
    pub tolerance: PatternTolerance,
    /// Total absolute deviation across all distances (lower is better).
    /// Recorded for rejected candidates too — near-misses feed the
    /// collision analysis.
    pub match_error: Millimeters,
}
