use tangible_objects_model::FrameCount;
use tangible_objects_model::PatternTolerance;

use crate::Percentiles;

/// One cell of the collision matrix: how well `pattern_name` explains the
/// geometry of frames labeled `label` (TRAINING.md 2.4).
#[derive(Debug)]
pub struct CollisionCell {
    /// Ground-truth label of the analyzed frames (the placed figure).
    pub label: String,
    /// Candidate pattern evaluated against the labeled geometry.
    pub pattern_name: String,
    /// Labeled frames where at least one candidate of this pattern existed.
    pub frames_tested: FrameCount,
    /// Frames where a candidate of this pattern actually matched - direct
    /// false-positive collisions.
    pub collided_frames: FrameCount,
    /// Percentiles of the per-frame minimal
    /// [`required_tolerance`](tangible_objects_model::CandidateReport::required_tolerance)
    /// - the tolerance at which this pattern would start matching the labeled
    ///   geometry.
    pub required: Option<Percentiles>,
    /// The candidate pattern's configured tolerance.
    pub tolerance: PatternTolerance,
}
