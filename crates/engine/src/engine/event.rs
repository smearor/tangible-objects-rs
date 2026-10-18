use tangible_objects_model::ContactCount;
use tangible_objects_model::FieldName;
use tangible_objects_model::FieldSetName;
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
    /// A field became occupied (debounced `placed` transition, count 0 -> n).
    FieldPlaced {
        /// Name of the owning field set.
        set: FieldSetName,
        /// Name of the occupied field.
        field: FieldName,
    },
    /// A field became empty (debounced `lifted` transition, count n -> 0).
    FieldLifted {
        /// Name of the owning field set.
        set: FieldSetName,
        /// Name of the vacated field.
        field: FieldName,
    },
    /// A field's debounced contact count changed (multi-contact fields).
    FieldCountChanged {
        /// Name of the owning field set.
        set: FieldSetName,
        /// Name of the field.
        field: FieldName,
        /// Confirmed contact count inside the field.
        count: ContactCount,
    },
}
