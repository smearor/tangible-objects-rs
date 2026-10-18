//! Profile similarity: pairwise fingerprint collision surface -
//! whether a reduced contact set of one figure can satisfy another
//! figure's (partial) fingerprint.

mod match_risk;
mod pair;
mod report;

pub use match_risk::MatchRisk;
pub use pair::SimilarityPair;
pub use report::SimilarityReport;
