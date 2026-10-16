use tangible_objects_model::FrameCount;
use tangible_objects_model::Millimeters;

/// Per-pattern accumulation state of the utilization report.
pub(crate) struct Accumulator {
    /// Frames with at least one matched full candidate.
    pub matched_frames: FrameCount,
    /// Frames where the pattern matched via a partial candidate only.
    pub partial_matched_frames: FrameCount,
    /// Frames carrying this pattern's ground-truth label.
    pub labeled_frames: FrameCount,
    /// Labeled frames without a matched candidate (figure present, missed).
    pub false_negative_frames: FrameCount,
    /// Deviation samples per distance index.
    pub deviations: Vec<Vec<Millimeters>>,
    /// Matched frames binned by `required_tolerance / tolerance` ratio.
    pub bands: [FrameCount; 4],
}

impl Default for Accumulator {
    fn default() -> Self {
        Self {
            matched_frames: FrameCount::new(0),
            partial_matched_frames: FrameCount::new(0),
            labeled_frames: FrameCount::new(0),
            false_negative_frames: FrameCount::new(0),
            deviations: Vec::new(),
            bands: [FrameCount::new(0); 4],
        }
    }
}
