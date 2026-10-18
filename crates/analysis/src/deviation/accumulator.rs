use tangible_objects_model::FrameCount;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternName;

/// Accumulator for one pattern while folding deviation samples.
///
/// Grows its `deviations`/`over`/`sole`/`blocker` vectors lazily to
/// the length of the first measured deviation vector - patterns with
/// different pin counts fold into their own accumulator.
pub(crate) struct PatternAcc {
    /// Profile name carried into the finished report.
    pub(crate) name: PatternName,
    /// Samples whose deviation vector satisfied the fingerprint.
    pub(crate) matched: FrameCount,
    /// Samples rejected by at least one over-tolerance distance.
    pub(crate) missed: FrameCount,
    /// Raw deviation samples per distance index (mm) - the percentile
    /// source for [`super::DistanceStats`].
    pub(crate) deviations: Vec<Vec<f64>>,
    /// Frames per index where the deviation exceeded its effective
    /// tolerance.
    pub(crate) over: Vec<FrameCount>,
    /// Frames per index where this index was the sole over-tolerance
    /// distance.
    pub(crate) sole: Vec<FrameCount>,
    /// Frames per index where this index was among the over-tolerance
    /// distances.
    pub(crate) blocker: Vec<FrameCount>,
}

impl PatternAcc {
    /// Creates an empty accumulator for a pattern.
    pub(crate) fn new(name: PatternName) -> Self {
        Self {
            name,
            matched: FrameCount::new(0),
            missed: FrameCount::new(0),
            deviations: Vec::new(),
            over: Vec::new(),
            sole: Vec::new(),
            blocker: Vec::new(),
        }
    }

    /// Folds one measured deviation vector into the per-index
    /// statistics; `tolerance_at` resolves the effective per-index
    /// tolerance (per-index list, else the scalar).
    pub(crate) fn push(
        &mut self,
        deviations: &[Millimeters],
        matched: bool,
        tolerance_at: impl Fn(usize) -> Millimeters,
    ) {
        if matched {
            self.matched += 1;
        } else {
            self.missed += 1;
        }

        let blockers: Vec<usize> = deviations
            .iter()
            .enumerate()
            .filter(|(index, deviation)| **deviation > tolerance_at(*index))
            .map(|(index, _)| index)
            .collect();

        for (index, deviation) in deviations.iter().enumerate() {
            if self.deviations.len() <= index {
                self.deviations.resize_with(index + 1, Vec::new);
                self.over.resize(index + 1, FrameCount::new(0));
                self.sole.resize(index + 1, FrameCount::new(0));
                self.blocker.resize(index + 1, FrameCount::new(0));
            }
            self.deviations[index].push(deviation.as_f64());
            if *deviation > tolerance_at(index) {
                self.over[index] += 1;
            }
            if blockers.len() == 1 && blockers[0] == index {
                self.sole[index] += 1;
            }
            if blockers.contains(&index) {
                self.blocker[index] += 1;
            }
        }
    }
}
