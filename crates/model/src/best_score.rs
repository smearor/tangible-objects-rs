use crate::MatchScore;

/// The best-scoring candidate seen so far during an enumeration -
/// `None` until the first candidate is considered.
///
/// Score-based ranking (TRAINING.md 3.2): the lowest score wins, so a
/// later candidate replaces the incumbent only when it strictly
/// improves. `S` is the ranking measure - `MatchScore` for
/// tolerance-normalized candidate selection (the matcher's
/// `PatternMatch` ranking), a raw `f64` residual for transform fits.
/// Shared by the engine's match selection and the deviation analysis'
/// best-subset evaluation.
#[derive(Debug)]
pub struct BestScore<T, S = MatchScore>(Option<(T, S)>);

impl<T, S> Default for BestScore<T, S> {
    /// Starts with no best - the first `consider` always wins.
    fn default() -> Self {
        Self(None)
    }
}

impl<T, S: PartialOrd> BestScore<T, S> {
    /// Keeps `value` when it scores strictly better than the current
    /// best.
    pub fn consider(&mut self, value: T, score: S) {
        let better = self
            .0
            .as_ref()
            .is_none_or(|(_, best_score)| score < *best_score);
        if better {
            self.0 = Some((value, score));
        }
    }

    /// Whether no candidate has been considered yet.
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    /// Consumes the selection and returns the winning value with its
    /// score.
    pub fn into_inner(self) -> Option<(T, S)> {
        self.0
    }

    /// Consumes the selection and returns the winning value.
    pub fn into_value(self) -> Option<T> {
        self.0.map(|(value, _)| value)
    }
}
