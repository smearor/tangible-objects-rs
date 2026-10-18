use crate::MatchError;

/// Result of a partial match: the reference subset that fit the reduced
/// contact set plus the total deviation it scored.
///
/// `kept_indices` preserves the reference ordering; the complement are
/// the missing pins - the completion path predicts their positions from
/// exactly this set.
#[derive(Debug, Clone)]
pub struct PartialMatch {
    /// Indices of the kept reference points, in reference order.
    pub kept_indices: Vec<usize>,
    /// Sum of absolute distance deviations between measured and
    /// reference fingerprint.
    pub match_error: MatchError,
}
