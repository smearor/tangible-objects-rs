//! Missing-pin completion: a partial match predicts where its missing
//! pins should be and claims a nearby unassigned touch.
//!
//! See `concepts/planned/TRACKING_STABILIZATION.md` - "Missing-Point
//! Completion".

mod affinity;
mod config;
mod mode;
mod smoothing;

pub use affinity::AffinityFactor;
pub use config::CompletionConfig;
pub use mode::CompletionMode;
pub use smoothing::SmoothingAlpha;
