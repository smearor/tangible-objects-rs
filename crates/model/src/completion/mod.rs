//! Missing-pin completion: a partial match predicts where its missing
//! pins should be and claims a nearby unassigned touch.
//!
//! See `concepts/planned/TRACKING_STABILIZATION.md` - "Missing-Point
//! Completion".

mod config;
mod mode;

pub use config::CompletionConfig;
pub use mode::CompletionMode;
