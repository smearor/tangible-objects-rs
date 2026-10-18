//! Tangible figures tracked across frames.
//!
//! [`TrackedTangible`] holds the smoothed pose of a figure that the
//! engine's tracker associates with consecutive [`crate::PatternMatch`]es.

mod count;
mod id;
mod tracked;

pub use count::TangibleCount;
pub use id::TangibleId;
pub use tracked::TrackedTangible;
