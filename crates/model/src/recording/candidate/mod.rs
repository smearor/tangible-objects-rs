//! Candidate-level recording types: the `CandidateReport` diagnostics record
//! plus the `CandidateCount` cardinality newtype.

mod count;
mod report;
mod score;

pub use count::CandidateCount;
pub use report::CandidateReport;
pub use score::MatchScore;
