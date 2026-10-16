use tangible_objects_model::FrameCount;
use tangible_objects_model::PatternId;

use crate::Percentiles;

/// Utilization statistics of one pattern across the recording.
#[derive(Debug)]
pub struct PatternUtilization {
    /// ID of the figure profile; `None` for label-only rows (a label that
    /// never produced a candidate, e.g. a figure whose pin count exceeded
    /// the recorded touch count — or a misspelled label).
    pub pattern_id: Option<PatternId>,
    /// Name of the figure profile (or the raw label for label-only rows).
    pub pattern_name: String,
    /// Frames with at least one matched full (non-partial) candidate.
    pub matched_frames: FrameCount,
    /// Frames where the pattern was matched by a partial candidate only.
    pub partial_matched_frames: FrameCount,
    /// Frames carrying this pattern's ground-truth label.
    pub labeled_frames: FrameCount,
    /// Labeled frames without any matched candidate for this pattern —
    /// the figure lay on the surface but was not detected.
    pub false_negative_frames: FrameCount,
    /// Deviation distribution per distance index (matched full candidates;
    /// index `i` corresponds to `sorted_distances[i]`).
    pub per_distance: Vec<Percentiles>,
    /// Matched frames binned by `required_tolerance / tolerance`:
    /// `[<25 %, 25–50 %, 50–75 %, 75–100 %]`.
    pub bands: [FrameCount; 4],
}
