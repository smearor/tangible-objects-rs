use tangible_objects_model::MatchScore;
use tangible_objects_model::PatternMatch;

/// The best-scoring candidate seen so far during combination
/// enumeration - `None` until the first candidate is considered.
///
/// Score-based ranking (TRAINING.md 3.2): the lowest normalized
/// deviation score wins, so a later candidate replaces the incumbent
/// only when it strictly improves.
#[derive(Debug, Default)]
pub(crate) struct BestMatch(Option<(PatternMatch, MatchScore)>);

impl BestMatch {
    /// Keeps `candidate` when it scores strictly better than the
    /// current best.
    pub(crate) fn consider(&mut self, candidate: PatternMatch, score: MatchScore) {
        match &self.0 {
            None => self.0 = Some((candidate, score)),
            Some((_, best_score)) if score < *best_score => {
                self.0 = Some((candidate, score));
            }
            _ => {}
        }
    }

    /// Whether no candidate has been considered yet.
    pub(crate) fn is_none(&self) -> bool {
        self.0.is_none()
    }

    /// Consumes the selection and returns the winning match.
    pub(crate) fn into_match(self) -> Option<PatternMatch> {
        self.0.map(|(candidate, _)| candidate)
    }
}
