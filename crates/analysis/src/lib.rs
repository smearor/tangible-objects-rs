//! Offline analysis of training recordings (TRAINING.md phase 2).
//!
//! [`Recording`] loads a JSONL artifact written by `--record`; the reports -
//! [`UtilizationReport`], [`ToleranceReport`], [`FlappyReport`] and
//! [`CollisionReport`] - turn the recorded candidates into tolerance
//! decisions. All evaluation happens offline, never in the hot detection
//! loop.

mod collision;
mod dataset;
mod deviation;
mod error;
mod flappy;
mod ml;
mod percentile;
mod recording;
mod scoring;
mod similarity;
mod tolerance;
mod utilization;

pub use collision::CollisionCell;
pub use collision::CollisionReport;
pub use dataset::Dataset;
pub use dataset::DatasetRow;
pub use deviation::DeviationReport;
pub use deviation::DistanceStats;
pub use deviation::PatternDeviation;
pub use error::AnalysisError;
pub use flappy::FlappyReport;
pub use flappy::LifetimeBucket;
pub use flappy::PartialShare;
pub use ml::LogisticBaseline;
pub use ml::MlReport;
pub use percentile::Percentiles;
pub use recording::Recording;
pub use scoring::ScoringReport;
pub use scoring::ThresholdRow;
pub use similarity::MatchRisk;
pub use similarity::SimilarityPair;
pub use similarity::SimilarityReport;
pub use tolerance::DEFAULT_SAFETY_FACTOR;
pub use tolerance::ToleranceReport;
pub use tolerance::ToleranceSuggestion;
pub use utilization::PatternUtilization;
pub use utilization::UtilizationReport;
