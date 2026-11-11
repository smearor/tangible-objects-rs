//! Spell training state - collects completed gesture episodes and
//! derives the template plus a corridor suggestion (SPELLS.md's
//! learn-by-repetition).

use tangible_objects_model::FingerCount;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Spell;
use tangible_objects_model::SpellArming;
use tangible_objects_model::SpellName;
use tangible_objects_model::StrokePolyline;
use tangible_objects_model::Vec2;

/// Repetitions a spell training captures before it derives template
/// and corridor - learn-by-repetition like the tangible training
/// (SPELLS.md).
pub const SPELL_TRAINING_REPETITIONS: usize = 3;

/// Floor under a suggested corridor in millimeters - training jitter
/// alone never produces a corridor tighter than this.
const MIN_TRAINED_CORRIDOR_MM: f64 = 5.0;

/// Margin over the worst repetition-vs-template score - the
/// suggestion should accept the next close-enough repetition, not
/// only the exact three it learned from.
const CORRIDOR_MARGIN: f64 = 1.3;

/// An in-progress spell training session (SPELLS.md). Completed
/// gesture episodes are captured until `SPELL_TRAINING_REPETITIONS`
/// land; the first episode becomes the template and the corridor is
/// derived from the repetitions' worst deviation from it.
pub(crate) struct SpellTraining {
    /// Name of the spell being trained.
    name: SpellName,
    /// Arming the finished spell carries - the trainer does not gate
    /// episodes on it (every completed episode counts).
    arming: SpellArming,
    /// Captured episodes, each a per-finger stroke list. Only
    /// episodes matching the first episode's stroke count count - a
    /// mistimed repetition doesn't corrupt the template.
    episodes: Vec<Vec<Vec<Vec2<Millimeters>>>>,
}

impl SpellTraining {
    /// Arms a training session for `name`; `arming` becomes the
    /// finished spell's trigger.
    pub(crate) fn new(name: SpellName, arming: SpellArming) -> Self {
        Self {
            name,
            arming,
            episodes: Vec::new(),
        }
    }

    /// Captures one completed episode - the per-finger stroke
    /// polylines in surface millimeters. Returns the finished
    /// [`Spell`] once `SPELL_TRAINING_REPETITIONS` consistent
    /// episodes landed; mismatched finger counts are skipped so the
    /// user can retry a flubbed repetition without restarting.
    pub(crate) fn capture(&mut self, strokes: Vec<Vec<Vec2<Millimeters>>>) -> Option<Spell> {
        if strokes.is_empty()
            || self
                .episodes
                .first()
                .is_some_and(|first| first.len() != strokes.len())
        {
            return None;
        }
        self.episodes.push(strokes);
        if self.episodes.len() < SPELL_TRAINING_REPETITIONS {
            return None;
        }
        Some(self.finish())
    }

    /// Template = first episode; corridor = worst
    /// repetition-vs-template match score scaled by
    /// `CORRIDOR_MARGIN` and floored - the same normalized-Fréchet
    /// metric `try_match` will apply at runtime.
    fn finish(&self) -> Spell {
        let template = &self.episodes[0];
        let probe = Spell {
            name: self.name.clone(),
            fingers: FingerCount::new(template.len()),
            corridor: Millimeters::new(f64::INFINITY),
            scale_invariant: true,
            enabled: true,
            strokes: template
                .iter()
                .map(|points| StrokePolyline::new(points.clone()))
                .collect(),
            arming: self.arming.clone(),
        };
        let worst = self.episodes[1..]
            .iter()
            .filter_map(|episode| probe.try_match(episode))
            .map(|score| score.as_f64())
            .fold(0.0_f64, f64::max);
        Spell {
            corridor: Millimeters::new((worst * CORRIDOR_MARGIN).max(MIN_TRAINED_CORRIDOR_MM)),
            ..probe
        }
    }
}
