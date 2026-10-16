//! Per-pair tolerance recommendations (TRAINING.md 2.2).

mod report;
mod suggestion;

pub use report::DEFAULT_SAFETY_FACTOR;
pub use report::ToleranceReport;
pub use suggestion::ToleranceSuggestion;
