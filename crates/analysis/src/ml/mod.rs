//! Baseline ML classifier (TRAINING.md 3.3, optional).
//!
//! Only pursued when the score sweep shows headroom; evaluation requires
//! labeled recordings.

mod accuracy;
mod classifier;
mod report;

pub use accuracy::Accuracy;
pub use classifier::LogisticBaseline;
pub use report::MlReport;
