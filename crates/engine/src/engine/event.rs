use tangible_objects_model::TrackedTangible;

/// Domain event emitted by the engine for a completed input frame.
///
/// Collected in [`crate::EngineFrame::events`]; consumed by interface crates
/// (GraphQL subscriptions, D-Bus signals, MCP notifications).
#[derive(Debug, Clone)]
pub enum EngineEvent {
    /// The set of active touch points changed since the previous frame.
    TouchPointsUpdated,
    /// A new tangible was detected and is now tracked.
    TangibleAdded(TrackedTangible),
    /// A tracked tangible's smoothed pose was updated by a match.
    TangibleMoved(TrackedTangible),
    /// A tracked tangible expired after exceeding its match-less frame budget.
    TangibleRemoved(TrackedTangible),
}
