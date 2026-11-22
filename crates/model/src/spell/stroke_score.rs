//! `StrokeScore` - one trajectory's entry in a spell-match
//! diagnostic: which template stroke it was assigned to and how far
//! it deviated.

use smearor_commons_geometry::Millimeters;

/// Per-stroke result of a spell match evaluation - the winning
/// finger→template assignment's index plus its discrete-Fréchet
/// deviation. Diagnostic surface for SPELLS.md's debug view; kept
/// out of the dispatch result so matching stays cheap when nobody
/// reads it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrokeScore {
    /// Index into [`crate::Spell::strokes`] - the template stroke
    /// this trajectory was assigned to by the best permutation.
    pub template: usize,
    /// Discrete-Fréchet distance between the normalized trajectory
    /// and the assigned template stroke (mm, reference extent).
    pub score: Millimeters,
}
