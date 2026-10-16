//! Offline analysis of training recordings (TRAINING.md phase 2).
//!
//! [`Recording`] loads a JSONL artifact written by `--record`; the reports —
//! [`UtilizationReport`], [`ToleranceReport`], [`FlappyReport`] and
//! [`CollisionReport`] — turn the recorded candidates into tolerance
//! decisions. All evaluation happens offline, never in the hot detection
//! loop.

mod collision;
mod error;
mod flappy;
mod percentile;
mod recording;
mod tolerance;
mod utilization;

pub use collision::CollisionCell;
pub use collision::CollisionReport;
pub use error::AnalysisError;
pub use flappy::FlappyReport;
pub use flappy::LifetimeBucket;
pub use flappy::PartialShare;
pub use percentile::Percentiles;
pub use recording::Recording;
pub use tolerance::DEFAULT_SAFETY_FACTOR;
pub use tolerance::ToleranceReport;
pub use tolerance::ToleranceSuggestion;
pub use utilization::PatternUtilization;
pub use utilization::UtilizationReport;
