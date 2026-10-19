use std::collections::HashSet;
use std::ops::Deref;
use std::ops::DerefMut;

use tangible_objects_model::FieldSet;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::TouchPointSlot;

use super::stage::Stage;
use super::stage_frame::ScopedMatches;
use super::stage_frame::StageFrame;
use tangible_objects_model::StageName;
use tangible_objects_model::TouchSet;

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
    /// Advances every stage's classification by one frame - each stage
    /// receives the `scoped` subset keyed by its own `scope()` plus the
    /// shared touches (EVENT_PIPELINE.md phase 4 input masking).
    pub fn classify(&mut self, scoped: &ScopedMatches, touches: TouchSet<'_>) {
        let empty = HashSet::new();
        for stage in &mut self.0 {
            let matched = scoped.get(stage.scope()).unwrap_or(&empty);
            stage.classify(&StageFrame { matched, touches });
        }
    }

    /// The stage keyed by `scope`, if present - the daemon ensures one
    /// per scope appearing on patterns.
    pub fn find_by_scope(&self, scope: &StageName) -> Option<&dyn Stage> {
        self.iter()
            .find(|stage| stage.scope().as_ref() == Some(scope))
            .map(|stage| &**stage)
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

    /// Propagates the current field sets to the field stage - other
    /// stages ignore it.
    pub fn set_field_sets(&mut self, sets: Vec<FieldSet>) {
        for stage in &mut self.0 {
            stage.set_fields(sets.clone());
        }
    }

    /// Whether a field stage is present.
    pub fn has_field_stage(&self) -> bool {
        self.iter().any(|stage| stage.is_field_stage())
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
