use serde::Deserialize;
use serde::Serialize;
use tangible_objects_model::FrameCount;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;

use super::DistanceStats;

/// Per-pattern deviation analysis.
#[derive(Debug, Serialize, Deserialize)]
pub struct PatternDeviation {
    /// Pattern the statistics belong to.
    pub pattern_id: PatternId,
    /// Profile name.
    pub pattern_name: PatternName,
    /// Frames with a matched full candidate.
    pub matched_frames: FrameCount,
    /// Frames with a full candidate that did not match.
    pub missed_frames: FrameCount,
    /// Per-distance-index statistics, in fingerprint order.
    pub distances: Vec<DistanceStats>,
}
