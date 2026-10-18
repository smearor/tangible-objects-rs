//! Tangible figures tracked across frames.
//!
//! [`TrackedTangible`] holds the smoothed pose of a figure that the
//! engine's tracker associates with consecutive [`crate::PatternMatch`]es.

mod count;
mod id;
mod tracked;
mod tracked_buf;
mod tracked_set;

pub use count::TangibleCount;
pub use id::TangibleId;
pub use tracked::TrackedTangible;
pub use tracked_buf::TrackedBuf;
pub use tracked_set::TrackedSet;
