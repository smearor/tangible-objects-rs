//! Per-distance deviation detail analysis: which measured distance
//! pushed a candidate over tolerance and how often - the causal view
//! behind match failures, computed over a whole recording.

mod accumulator;
mod best_subset;
mod distance_stats;
mod pattern_deviation;
mod report;

pub(crate) use accumulator::PatternAcc;
pub use distance_stats::DistanceStats;
pub use pattern_deviation::PatternDeviation;
pub use report::DeviationReport;
