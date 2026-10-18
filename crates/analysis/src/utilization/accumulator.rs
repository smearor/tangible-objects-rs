use tangible_objects_model::FrameCount;

use crate::deviation_samples::DeviationSamples;
use crate::utilization::RatioBands;

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
    /// Deviation samples per distance index - from the labeled pattern's
    /// best candidate per labeled frame, matched or not.
    pub deviation_samples: Vec<DeviationSamples>,
    /// Labeled presence frames binned by `required_tolerance / tolerance`
    /// ratio (`< 1.0` - within the configured tolerance).
    pub bands: RatioBands,
    /// Labeled presence frames whose best candidate would have needed a
    /// tolerance above the configured one (ratio `>= 1.0`) - the misses
    /// the bands cannot represent.
    pub over_tolerance_frames: FrameCount,
}

impl Default for Accumulator {
    fn default() -> Self {
        Self {
            matched_frames: FrameCount::new(0),
            partial_matched_frames: FrameCount::new(0),
            labeled_frames: FrameCount::new(0),
            false_negative_frames: FrameCount::new(0),
            deviation_samples: Vec::new(),
            bands: Default::default(),
            over_tolerance_frames: FrameCount::new(0),
        }
    }
}
