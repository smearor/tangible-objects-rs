use async_graphql::Enum;

/// Discriminator for [`super::TangibleEvent`] payloads.
#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum EventKind {
    /// The set of active touch points changed.
    TouchPointsUpdated,
    /// A new tangible was detected and is now tracked.
    TangibleAdded,
    /// A tracked tangible's pose changed.
    TangibleMoved,
    /// A tracked tangible expired.
    TangibleRemoved,
    /// All mandatory set members became present (TOS-ADD).
    SetAdded,
    /// The last mandatory set member disappeared (TOS-REM).
    SetRemoved,
    /// A member tangible appeared in an active set (TOS-OBJ-ADD).
    ObjectAdded,
    /// A member tangible disappeared (TOS-OBJ-REM).
    ObjectRemoved,
    /// A member tangible's pose changed in an active set (TOS-OBJ-MOV).
    ObjectMoved,
    /// An occupancy field became occupied (debounced placed).
    FieldPlaced,
    /// An occupancy field became empty (debounced lifted).
    FieldLifted,
    /// An occupancy field's debounced contact count changed.
    FieldCountChanged,
}
