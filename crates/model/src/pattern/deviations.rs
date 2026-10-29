use std::ops::Deref;

use serde::Deserialize;
use serde::Serialize;

use crate::Deviation;
use crate::MatchError;
use crate::MatchScore;
use crate::Millimeters;

/// Per-index absolute deviations `|measured[i] - reference[i]|`
/// between a measured fingerprint and a reference fingerprint -
/// parallel to [`crate::Distances`], not a fingerprint itself.
///
/// A deviations vector has no ordering invariant (its order is the
/// fingerprint's distance index order) and its values are deltas, not
/// distances - the distinction a `Distances`/`Deviations` split makes
/// visible.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Deviations(pub Vec<Deviation>);

impl Deviations {
    /// The deviation slice.
    pub fn as_slice(&self) -> &[Deviation] {
        &self.0
    }

    /// Highest normalized score - `max_i deviation[i] / tolerance(i)`.
    ///
    /// `tolerance_at` resolves the tolerance of deviation `i` (per-index
    /// `Tolerances` or a widened partial constant); `0.0` on empty input.
    /// `<= 1.0` means the candidate matched.
    pub fn max_score(&self, tolerance_at: impl Fn(usize) -> Millimeters) -> MatchScore {
        self.iter()
            .enumerate()
            .map(|(i, dev)| MatchScore::normalized(dev.as_millimeters(), tolerance_at(i)))
            .reduce(MatchScore::max)
            .unwrap_or_default()
    }

    /// Total absolute deviation across all indices - the candidate's
    /// match error (lower is better).
    pub fn total(&self) -> MatchError {
        MatchError::new(Millimeters::new(
            self.0.iter().map(|deviation| deviation.as_f64()).sum(),
        ))
    }
}

impl From<Vec<Deviation>> for Deviations {
    /// A deviation vector IS a deviations set.
    fn from(deviations: Vec<Deviation>) -> Self {
        Self(deviations)
    }
}

impl Deref for Deviations {
    type Target = [Deviation];

    /// Deviations view their slice transparently.
    fn deref(&self) -> &[Deviation] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Deviations;
    use crate::Deviation;
    use crate::Millimeters;

    /// Per-index tolerances: `deviation[i] / tolerance(i)` - the worst
    /// (highest) normalized score wins.
    #[test]
    fn max_score_uses_per_index_tolerance() {
        let deviations = Deviations::from(vec![
            Deviation::new(Millimeters::new(5.0)),
            Deviation::new(Millimeters::new(10.0)),
            Deviation::new(Millimeters::new(2.0)),
        ]);
        // Index-wise: 5/10, 10/20, 2/0.5 → 0.5, 0.5, 4.0 → max = 4.0.
        let score = deviations.max_score(|i| match i {
            0 => Millimeters::new(10.0),
            1 => Millimeters::new(20.0),
            _ => Millimeters::new(0.5),
        });
        assert_eq!(score.as_f64(), 4.0);
    }

    /// A constant tolerance closure is the partial-matching case:
    /// `dev / tol` against the same widened baseline.
    #[test]
    fn max_score_constant_tolerance() {
        let deviations = Deviations::from(vec![
            Deviation::new(Millimeters::new(3.0)),
            Deviation::new(Millimeters::new(9.0)),
            Deviation::new(Millimeters::new(4.5)),
        ]);
        let score = deviations.max_score(|_| Millimeters::new(4.5));
        assert_eq!(score.as_f64(), 2.0);
    }

    /// An empty vector aggregates to the `max` identity `0.0`.
    #[test]
    fn max_score_empty_is_zero() {
        let score = Deviations::default().max_score(|_| Millimeters::new(1.0));
        assert_eq!(score.as_f64(), 0.0);
    }
}
