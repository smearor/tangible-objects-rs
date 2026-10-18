use tangible_objects_model::Deviations;
use tangible_objects_model::MatchScore;

/// The best-fitting contact subset found in one frame - the subset's
/// deviations and its normalized score.
///
/// Best-subset evaluation counts only the geometrically closest subset
/// per frame (lowest score) - foreign contacts like hands or other
/// objects must not inflate a figure's deviation statistics.
#[derive(Debug, Default)]
pub(crate) struct BestSubset(Option<(Deviations, MatchScore)>);

impl BestSubset {
    /// Keeps `deviations` when the subset scores strictly better than
    /// the current best.
    pub(crate) fn consider(&mut self, deviations: Deviations, score: MatchScore) {
        match &self.0 {
            None => self.0 = Some((deviations, score)),
            Some((_, best_score)) if score < *best_score => {
                self.0 = Some((deviations, score));
            }
            _ => {}
        }
    }

    /// Consumes the selection and returns the best subset's deviations
    /// and score.
    pub(crate) fn into_eval(self) -> Option<(Deviations, MatchScore)> {
        self.0
    }
}
