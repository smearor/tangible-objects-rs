//! `Spell` - a named multi-finger gesture: per-finger template
//! strokes plus a matching corridor (SPELLS.md).

use serde::Deserialize;
use serde::Serialize;

use super::FingerCount;
use super::SpellArming;
use super::SpellName;
use super::StrokePolyline;
use crate::Millimeters;
use crate::Vec2;

/// Reference extent scale-normalized strokes are matched at.
pub const REFERENCE_EXTENT_MM: f64 = 60.0;

/// A trainable, registrable gesture definition.
///
/// Serialized from `spells.d/*.toml`:
///
/// ```toml
/// name = "Pentagram"
/// fingers = 2
/// corridor = 12.0
/// scale_invariant = true
/// [[strokes]]
/// points = [[0.0, 0.0], [30.0, 40.0], [60.0, 0.0]]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spell {
    /// Identity - addresses the spell for unregister and the store.
    pub name: SpellName,
    /// Simultaneous contacts the gesture uses - must equal
    /// `strokes.len()` (validated on load/match).
    pub fingers: FingerCount,
    /// Max per-stroke deviation (discrete Fréchet) in millimeters.
    pub corridor: Millimeters,
    /// Whether the gesture may be drawn larger/smaller than the
    /// template (scale-normalizes both sides before matching).
    #[serde(default = "default_scale_invariant")]
    pub scale_invariant: bool,
    /// Whether the spell participates in matching
    /// (`enabled = false` in TOML keeps it registered but inert).
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Per-finger template strokes in pattern space (origin at the
    /// first point, mm).
    pub strokes: Vec<StrokePolyline>,
    /// How the gesture is initiated (SPELLS.md's Einläutung
    /// phase) - `arming = "free"` defers to the prefix-gated
    /// fallback.
    #[serde(default = "default_arming")]
    pub arming: SpellArming,
}

/// Spells are scale-invariant by default (SPELLS.md).
fn default_scale_invariant() -> bool {
    true
}

/// Default enabled state (`enabled = true`).
fn default_enabled() -> bool {
    true
}

/// Default arming - `free` keeps old TOMLs working (they gain an
/// explicit `[arming]` section when the trigger is known).
fn default_arming() -> SpellArming {
    SpellArming::Free
}

impl Spell {
    /// The best score across all finger→stroke assignments - the
    /// worst per-stroke Fréchet after normalization; `None` when the
    /// trajectory count doesn't equal `fingers`.
    ///
    /// `trajectories` are live stroke polylines in surface
    /// millimeters. A score ≤ `corridor` is a match.
    pub fn try_match(&self, trajectories: &[Vec<Vec2<Millimeters>>]) -> Option<Millimeters> {
        if !self.enabled || trajectories.len() != self.fingers.as_usize() {
            return None;
        }
        let reference = Millimeters::new(REFERENCE_EXTENT_MM);
        let templates: Vec<StrokePolyline> = self
            .strokes
            .iter()
            .map(|stroke| {
                stroke
                    .resampled()
                    .normalized(reference, self.scale_invariant)
            })
            .collect();
        let normalized: Vec<StrokePolyline> = trajectories
            .iter()
            .map(|traj| {
                StrokePolyline::new(traj.clone())
                    .resampled()
                    .normalized(reference, self.scale_invariant)
            })
            .collect();
        // k! assignments - tiny for k <= 5 fingers.
        let mut perm: Vec<usize> = (0..templates.len()).collect();
        let mut best = f64::INFINITY;
        permute(&mut perm, 0, &mut |p| {
            let score = p
                .iter()
                .enumerate()
                .map(|(finger, &stroke_index)| {
                    normalized[finger]
                        .frechet(&templates[stroke_index])
                        .as_f64()
                })
                .fold(0.0_f64, f64::max);
            best = best.min(score);
        });
        Some(Millimeters::new(best))
    }
}

/// Heap's algorithm - visits every permutation of `perm`, calling
/// `visit` on each (assignment problem is tiny; no Hungarian needed).
fn permute(perm: &mut Vec<usize>, k: usize, visit: &mut impl FnMut(&[usize])) {
    if k == perm.len() {
        visit(perm);
        return;
    }
    for i in k..perm.len() {
        perm.swap(k, i);
        permute(perm, k + 1, visit);
        perm.swap(k, i);
    }
}

#[cfg(test)]
mod tests {
    use super::Spell;
    use super::SpellArming;
    use super::SpellName;
    use crate::FingerCount;
    use crate::Millimeters;
    use crate::StrokePolyline;
    use crate::Vec2;

    fn v_shape() -> Spell {
        Spell {
            name: SpellName::new("V"),
            fingers: FingerCount::new(1),
            corridor: Millimeters::new(15.0),
            scale_invariant: true,
            enabled: true,
            strokes: vec![StrokePolyline::new(vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(30.0, 40.0),
                Vec2::new(60.0, 0.0),
            ])],
            arming: SpellArming::Free,
        }
    }

    #[test]
    fn matching_v_scores_under_corridor() {
        // Same V drawn shifted and 1.5x bigger.
        let traj = vec![
            Vec2::new(100.0, 100.0),
            Vec2::new(145.0, 160.0),
            Vec2::new(190.0, 100.0),
        ];
        let score = v_shape().try_match(&[traj]).unwrap();
        assert!(score.as_f64() < 15.0, "score {}", score.as_f64());
    }

    #[test]
    fn wrong_shape_scores_over_corridor() {
        let traj = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(30.0, 0.0),
            Vec2::new(60.0, 0.0),
        ];
        assert!(v_shape().try_match(&[traj]).unwrap().as_f64() > 15.0);
    }

    #[test]
    fn wrong_finger_count_rejects() {
        let traj = vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)];
        assert!(v_shape().try_match(&[traj.clone(), traj]).is_none());
    }

    #[test]
    fn disabled_spell_rejects() {
        let mut spell = v_shape();
        spell.enabled = false;
        let traj = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(30.0, 40.0),
            Vec2::new(60.0, 0.0),
        ];
        assert!(spell.try_match(&[traj]).is_none());
    }

    #[test]
    fn two_finger_spell_matches_independent_of_finger_order() {
        let mut spell = v_shape();
        spell.fingers = FingerCount::new(2);
        spell.strokes = vec![
            StrokePolyline::new(vec![Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0)]),
            StrokePolyline::new(vec![Vec2::new(0.0, 0.0), Vec2::new(0.0, 50.0)]),
        ];
        // Finger order swapped vs template order - assignment must
        // find the right pairing.
        let horizontal = vec![Vec2::new(10.0, 10.0), Vec2::new(60.0, 10.0)];
        let vertical = vec![Vec2::new(80.0, 20.0), Vec2::new(80.0, 70.0)];
        let score = spell
            .try_match(&[vertical.clone(), horizontal.clone()])
            .unwrap();
        assert!(score.as_f64() < 5.0, "score {}", score.as_f64());
        let score2 = spell.try_match(&[horizontal, vertical]).unwrap();
        assert!(score2.as_f64() < 5.0);
    }
}
