//! Candidate-level recording types: the `CandidateReport` diagnostics record
//! plus the `CandidateCount` cardinality newtype.

mod count;
mod report;

pub use count::CandidateCount;
pub use report::CandidateReport;
