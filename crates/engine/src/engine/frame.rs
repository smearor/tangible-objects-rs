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
}
