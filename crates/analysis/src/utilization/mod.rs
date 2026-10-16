//! Tolerance-utilization analysis (TRAINING.md 2.1).

mod accumulator;
mod pattern;
mod report;

pub use pattern::PatternUtilization;
pub use report::UtilizationReport;

pub(crate) use accumulator::Accumulator;
