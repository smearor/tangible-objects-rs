//! Accumulated recognition metrics of a session.
//!
//! [`SessionMetrics`] folds every published frame into counters -
//! uptime, re-ID churn, match quality and per-slot contact stability -
//! replacing the ad-hoc post-analysis previously run on `.jsonl`
//! recordings and enabling automated test series read back over the
//! interfaces.

mod pattern;
mod session;

pub use pattern::PatternMetrics;
pub use session::SessionMetrics;
