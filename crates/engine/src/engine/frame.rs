use tangible_objects_model::CandidateCount;
use tangible_objects_model::CandidateReport;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::TouchPoint;

use super::EngineEvent;

/// Result of one completed input frame (`SYN_REPORT` boundary).
///
/// Returned by [`crate::Engine::process`] whenever a frame was evaluated.
#[derive(Debug, Clone)]
pub struct EngineFrame {
    /// All pattern matches detected in this frame.
    pub matches: Vec<PatternMatch>,
    /// Active touch points at frame end, sorted by slot.
    pub touches: Vec<TouchPoint>,
    /// Domain events produced by this frame (touch changes, tangible
    /// added/moved/removed), in emission order.
    pub events: Vec<EngineEvent>,
    /// Diagnostics: every candidate combination tested in this frame -
    /// empty unless the frame was produced by
    /// [`Engine::process_with_diagnostics`](crate::Engine::process_with_diagnostics).
    pub candidates: Vec<CandidateReport>,
    /// Whether the generic matching pass ran this frame
    /// (SKIP_UNCHANGED.md): `false` on idle-board frames where the
    /// contact set was unchanged, no spawn was pending and no matcher
    /// input changed.
    pub ran_generic: bool,
    /// Candidate combinations evaluated in this frame's generic pass -
    /// the pruning ratio is observable as this count versus the
    /// unpruned C(touches, k) per pattern (SPATIAL_CLUSTERING.md).
    pub candidates_tested: CandidateCount,
}
