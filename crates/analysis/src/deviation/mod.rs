//! Per-distance deviation detail analysis: which measured distance
//! pushed a candidate over tolerance and how often - the causal view
//! behind match failures, computed over a whole recording.

mod report;

pub use report::DeviationReport;
pub use report::DistanceStats;
pub use report::PatternDeviation;
