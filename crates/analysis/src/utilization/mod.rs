//! Tolerance-utilization analysis (TRAINING.md 2.1).

mod accumulator;
mod pattern;
mod ratio_bands;
mod report;

pub use pattern::PatternUtilization;
pub use ratio_bands::RatioBands;
pub use report::UtilizationReport;

pub(crate) use accumulator::Accumulator;
