use tangible_objects_model::PatternMatch;

/// Result of one completed input frame (`SYN_REPORT` boundary).
///
/// Returned by [`crate::Engine::process`] whenever a frame was evaluated.
#[derive(Debug, Clone)]
pub struct EngineFrame {
    /// All pattern matches detected in this frame.
    pub matches: Vec<PatternMatch>,
}
