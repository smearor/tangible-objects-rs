use tangible_objects_model::FrameCount;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternTolerance;

/// Per-cell accumulation state of the collision matrix.
pub(crate) struct Accumulator {
    /// Labeled frames where at least one candidate of this pattern existed.
    pub frames_tested: FrameCount,
    /// Frames where the pattern's best candidate matched (false positive).
    pub collided_frames: FrameCount,
    /// Per-frame minimal
    /// [`required_tolerance`](tangible_objects_model::CandidateReport::required_tolerance)
    /// samples.
    pub required: Vec<Millimeters>,
    /// The candidate pattern's configured tolerance.
    pub tolerance: PatternTolerance,
}

impl Accumulator {
    /// Starts accumulating for a cell; the pattern tolerance is fixed for the
    /// whole session and taken from the first observed candidate.
    pub fn new(tolerance: PatternTolerance) -> Self {
        Self {
            frames_tested: FrameCount::new(0),
            collided_frames: FrameCount::new(0),
            required: Vec::new(),
            tolerance,
        }
    }
}
