use std::ops::Deref;

use serde::Serialize;

use tangible_objects_model::Deviation;

/// Observed deviation samples of one distance index - every frame's
/// `deviations[index]` collected into one column.
///
/// One sample column per fingerprint index: `samples[i]` holds the
/// per-frame deviations of distance `i`, feeding percentile and
/// tolerance statistics. Not a [`tangible_objects_model::Deviations`]
/// vector - the column crosses frames, a `Deviations` is one frame's
/// per-index vector.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(transparent)]
pub(crate) struct DeviationSamples(pub Vec<Deviation>);

impl DeviationSamples {
    /// Records one observed deviation.
    pub(crate) fn push(&mut self, deviation: Deviation) {
        self.0.push(deviation);
    }
}

impl Deref for DeviationSamples {
    type Target = [Deviation];

    /// Sample columns view their slice transparently.
    fn deref(&self) -> &[Deviation] {
        &self.0
    }
}
