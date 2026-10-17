use tangible_objects_model::CandidateCount;

/// Partial-match share of one pattern (TRAINING.md 2.3).
///
/// A high share indicates unstable contacts at figure level - the pattern
/// only matched because one pin was allowed to drop out. Invariant:
/// `partial_matches <= matched_total`.
#[derive(Debug)]
pub struct PartialShare {
    /// Name of the figure profile.
    pub pattern_name: String,
    /// Matched partial candidates.
    pub partial_matches: CandidateCount,
    /// All matched candidates (full + partial).
    pub matched_total: CandidateCount,
}

impl PartialShare {
    /// Share of partial matches among all matched candidates, 0.0–1.0.
    pub fn fraction(&self) -> f64 {
        if self.matched_total.as_u64() == 0 {
            0.0
        } else {
            self.partial_matches.as_u64() as f64 / self.matched_total.as_u64() as f64
        }
    }
}
