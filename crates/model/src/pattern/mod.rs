//! Geometric figure patterns and their matching within a frame.
//!
//! [`TangiblePattern`] is the runtime pattern loaded from a profile,
//! [`PatternMatch`] a successful detection within a frame, [`PatternId`]
//! the runtime identifier and [`PatternTolerance`] the matching tolerance.

mod candidate_diagnostics;
mod correspondences;
mod deviation;
mod deviations;
mod distance_multiset;
mod distances;
mod id;
mod r#match;
mod match_error;
mod matcher;
mod name;
mod partial_match;
mod partial_reference;
mod pattern_buf;
mod pattern_set;
mod pin_count;
mod ref_index;
mod ref_indices;
mod ref_points;
mod rigid_transform;
mod rotation_fit;
mod tangible;
mod tolerance;
mod tolerances;

pub use candidate_diagnostics::CandidateDiagnostics;
pub use candidate_diagnostics::NoDiagnostics;
pub use correspondences::Correspondences;
pub use deviation::Deviation;
pub use deviations::Deviations;
pub use distance_multiset::DistanceMultiset;
pub use distances::Distances;
pub use id::PatternId;
pub use r#match::PatternMatch;
pub use match_error::MatchError;
pub use name::PatternName;
pub use partial_match::PartialMatch;
pub use partial_reference::PartialReference;
pub use pattern_buf::PatternBuf;
pub use pattern_set::PatternSet;
pub use pin_count::PinCount;
pub use ref_index::RefIndex;
pub use ref_indices::RefIndices;
pub use ref_points::RefPoints;
pub use rigid_transform::RigidTransform;
pub use rotation_fit::RotationFit;
pub use tangible::TangiblePattern;
pub use tolerance::PatternTolerance;
pub use tolerances::PatternTolerances;
