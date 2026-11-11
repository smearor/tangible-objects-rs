//! `SpellMatched` - the dispatched result of a completed gesture:
//! which spell matched, where on the surface, and how well.

use serde::Deserialize;
use serde::Serialize;

use super::SpellName;
use crate::Millimeters;
use crate::Vec2;

/// A successfully recognized spell - the payload of
/// `LifecycleEvent::SpellMatched` and the gesture tracker's
/// dispatch result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpellMatched {
    /// The spell that matched.
    pub name: SpellName,
    /// Surface position of the gesture centroid (mm) - the action's
    /// natural anchor point for menus/effects.
    pub center: Vec2<Millimeters>,
    /// Match score - the worst per-stroke Fréchet deviation of the
    /// winning assignment (≤ the spell's corridor by definition).
    pub score: Millimeters,
}
