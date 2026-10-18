//! Geometric figure patterns and their matching within a frame.
//!
//! [`TangiblePattern`] is the runtime pattern loaded from a profile,
//! [`PatternMatch`] a successful detection within a frame, [`PatternId`]
//! the runtime identifier and [`PatternTolerance`] the matching tolerance.

mod id;
mod r#match;
mod name;
mod tangible;
mod tolerance;

pub use id::PatternId;
pub use r#match::PatternMatch;
pub use name::PatternName;
pub use tangible::TangiblePattern;
pub use tolerance::PatternTolerance;
