//! `SpellDiagnostics` - a completed gesture episode's match
//! evaluation against one spell: per-stroke Fréchet scores, the
//! winning finger assignment and the corridor verdict.

use smearor_commons_geometry::Millimeters;

use super::SpellName;
use super::StrokeScore;

/// Debug view into a match evaluation (SPELLS.md): what the matcher
/// computed for a completed episode - the spell evaluated, its
/// corridor, the best assignment's per-stroke deviations and whether
/// they landed inside the corridor.
///
/// The tracker retains the *best* candidate's diagnostics per
/// episode, so a rejected gesture still reports how far it missed.
#[derive(Debug, Clone, PartialEq)]
pub struct SpellDiagnostics {
    /// The spell this evaluation ran against.
    pub name: SpellName,
    /// The spell's configured corridor - `score <= corridor` is a
    /// match.
    pub corridor: Millimeters,
    /// The winning assignment's worst per-stroke deviation (mm) -
    /// the same value [`crate::Spell::try_match`] returns.
    pub score: Millimeters,
    /// Whether `score` landed inside `corridor`.
    pub matched: bool,
    /// Per-trajectory results ordered by episode stroke index -
    /// each entry names the assigned template stroke and its
    /// Fréchet deviation.
    pub strokes: Vec<StrokeScore>,
}
