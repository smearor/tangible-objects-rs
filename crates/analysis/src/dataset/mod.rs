//! Labeled dataset export (TRAINING.md 3.3).
//!
//! One JSONL row per recorded candidate - normalized deviation features plus
//! ground-truth target - for external ML tooling.

mod jsonl;
mod row;

pub use jsonl::Dataset;
pub use row::DatasetRow;
