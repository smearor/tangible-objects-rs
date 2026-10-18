use serde::Serialize;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Millimeters;

/// Per-distance-index deviation statistics of one pattern over a
/// recording - the causal "why did the match fail" answer.
///
/// `index i` refers to the pattern's `sorted_distances[i]`, i.e. a
/// concrete pin pair; a high `blocker` count pinpoints the unstable
/// pin pair that most often pushed the deviation over tolerance.
#[derive(Debug, Serialize)]
pub struct DistanceStats {
    /// Distance index into the pattern's sorted fingerprint.
    pub index: usize,
    /// Frames this index was measured (full candidates only).
    pub samples: FrameCount,
    /// Mean absolute deviation in millimeters.
    pub mean: Millimeters,
    /// Worst observed deviation in millimeters.
    pub max: Millimeters,
    /// 99th percentile deviation in millimeters.
    pub p99: Millimeters,
    /// Frames where this index exceeded its effective tolerance
    /// (per-index `tolerances[i]` when configured, else the scalar).
    pub over_tolerance: FrameCount,
    /// Frames where this index was the **only** over-tolerance
    /// distance - the direct cause of the miss.
    pub sole_blocker: FrameCount,
    /// Frames where this index was among the over-tolerance
    /// distances.
    pub blocker: FrameCount,
}
