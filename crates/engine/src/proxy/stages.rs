use std::collections::HashSet;
use std::ops::Deref;
use std::ops::DerefMut;

use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::TouchPointSlot;

use super::stage::Stage;
use super::stage_frame::StageFrame;

/// Owned stage chain of a pipeline (`Vec<Box<dyn Stage>>`).
///
/// Carries both access directions the emit boundary needs: read-side
/// merging (`merged_*`, `suppresses`, `was_suppressed`,
/// `suppressed_slots`) through `&self` and mutation (`classify`,
/// `release_slot`, `set_mode`) through `&mut self` - one borrow serves
/// both, so no view juggling is needed inside `finish`.
pub(crate) struct Stages(pub Vec<Box<dyn Stage>>);

// Read-side merge - the boundary's view over all stages.
impl Stages {
    /// The mechanism's effective policy across all stages:
    /// `TangiblesOnly` and `Guarded` are mechanism-visible (`Guarded`
    /// withholds, `TangiblesOnly` drains); `Immediate`/`Gated`/`Off`
    /// differ only in stage-side classification, not mechanism, so the
    /// first stage's mode suffices.
    pub fn merged_mode(&self) -> FilterMode {
        if self
            .iter()
            .any(|stage| stage.mode() == FilterMode::TangiblesOnly)
        {
            return FilterMode::TangiblesOnly;
        }
        if self.iter().any(|stage| stage.mode() == FilterMode::Guarded) {
            return FilterMode::Guarded;
        }
        self.first().map_or(FilterMode::default(), |s| s.mode())
    }

    /// The gate used for guarded-mode release - the gate of the (first)
    /// guarded stage, else any stage's.
    pub fn merged_gate(&self) -> FrameCount {
        self.iter()
            .find(|stage| stage.mode() == FilterMode::Guarded)
            .or_else(|| self.first())
            .map_or(FrameCount::new(0), |s| s.stability_gate())
    }

    /// Whether any stage suppresses `slot` under its active policy.
    pub fn suppresses(&self, slot: TouchPointSlot) -> bool {
        self.iter().any(|stage| stage.suppresses(slot))
    }

    /// Whether any stage holds `slot` in its suppressed set - used by the
    /// guarded-release logic.
    pub fn was_suppressed(&self, slot: TouchPointSlot) -> bool {
        self.iter().any(|stage| stage.was_suppressed(slot))
    }

    /// Union of every stage's suppressed slots - deduplicated: a slot
    /// suppressed by two stages must retract only once.
    pub fn suppressed_slots(&self) -> impl Iterator<Item = TouchPointSlot> + '_ {
        self.iter()
            .flat_map(|stage| stage.suppressed_slots())
            .collect::<HashSet<_>>()
            .into_iter()
    }
}

// Mutation side - per-frame classification and slot lifecycle.
impl Stages {
    /// Advances every stage's classification by one frame - `frame`
    /// carries the matched slots and active touches (unscoped until
    /// stage scoping lands).
    pub fn classify(&mut self, frame: &StageFrame<'_>) {
        for stage in &mut self.0 {
            stage.classify(frame);
        }
    }

    /// Releases `slot` on every stage - a lifting contact frees it for a
    /// new tangible regardless of which stage suppressed it.
    pub fn release_slot(&mut self, slot: TouchPointSlot) {
        for stage in &mut self.0 {
            stage.release(slot);
        }
    }

    /// Switches the suppression policy on every stage (`SetFilterMode`
    /// control command - unscoped until per-stage control lands; stages
    /// without a `FilterMode` concept ignore it).
    pub fn set_mode(&mut self, mode: FilterMode, stability: Option<FrameCount>) {
        for stage in &mut self.0 {
            stage.set_mode(mode, stability);
        }
    }
}

impl Deref for Stages {
    type Target = Vec<Box<dyn Stage>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Stages {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Box<dyn Stage>>> for Stages {
    fn from(stages: Vec<Box<dyn Stage>>) -> Self {
        Self(stages)
    }
}
