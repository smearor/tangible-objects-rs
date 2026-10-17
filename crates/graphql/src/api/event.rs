use async_graphql::SimpleObject;
use tangible_objects_engine::EngineEvent;
use tangible_objects_model::SetAction;
use tangible_objects_model::TangibleSetId;
use tangible_objects_model::TrackedTangible;
use tangible_objects_runtime::DomainEvent;

use super::EventKind;

/// Flattened domain event delivered by the `events` subscription.
///
/// `kind` discriminates the payload: `tangible` carries the affected object
/// for `TANGIBLE_*`/`OBJECT_*` kinds, `setId` for `SET_*`/`OBJECT_*` kinds,
/// `field`/`fieldSet`/`count` for `FIELD_*` kinds.
#[derive(Debug, Clone, SimpleObject)]
pub struct TangibleEvent {
    /// What happened.
    pub kind: EventKind,
    /// Affected tangible for `TANGIBLE_*` and `OBJECT_*` events.
    pub tangible: Option<TrackedTangible>,
    /// Affected set for `SET_*` and `OBJECT_*` events.
    pub set_id: Option<TangibleSetId>,
    /// Field-set name for `FIELD_*` events.
    pub field_set: Option<String>,
    /// Field name for `FIELD_*` events.
    pub field: Option<String>,
    /// Debounced contact count for `FIELD_COUNT_CHANGED` events.
    pub count: Option<i32>,
}

impl From<&EngineEvent> for TangibleEvent {
    fn from(event: &EngineEvent) -> Self {
        let (kind, tangible, field_set, field, count) = match event {
            EngineEvent::TouchPointsUpdated => {
                (EventKind::TouchPointsUpdated, None, None, None, None)
            }
            EngineEvent::TangibleAdded(tangible) => (
                EventKind::TangibleAdded,
                Some(tangible.clone()),
                None,
                None,
                None,
            ),
            EngineEvent::TangibleMoved(tangible) => (
                EventKind::TangibleMoved,
                Some(tangible.clone()),
                None,
                None,
                None,
            ),
            EngineEvent::TangibleRemoved(tangible) => (
                EventKind::TangibleRemoved,
                Some(tangible.clone()),
                None,
                None,
                None,
            ),
            EngineEvent::FieldPlaced { set, field } => (
                EventKind::FieldPlaced,
                None,
                Some(set.clone()),
                Some(field.clone()),
                None,
            ),
            EngineEvent::FieldLifted { set, field } => (
                EventKind::FieldLifted,
                None,
                Some(set.clone()),
                Some(field.clone()),
                None,
            ),
            EngineEvent::FieldCountChanged { set, field, count } => (
                EventKind::FieldCountChanged,
                None,
                Some(set.clone()),
                Some(field.clone()),
                Some(count.as_u32() as i32),
            ),
        };
        Self {
            kind,
            tangible,
            set_id: None,
            field_set,
            field,
            count,
        }
    }
}

impl From<&SetAction> for TangibleEvent {
    fn from(action: &SetAction) -> Self {
        let (kind, set_id, tangible) = match action {
            SetAction::SetAdded(id) => (EventKind::SetAdded, *id, None),
            SetAction::SetRemoved(id) => (EventKind::SetRemoved, *id, None),
            SetAction::ObjectAdded(id, tangible) => (EventKind::ObjectAdded, *id, Some(tangible)),
            SetAction::ObjectRemoved(id, tangible) => {
                (EventKind::ObjectRemoved, *id, Some(tangible))
            }
            SetAction::ObjectMoved(id, tangible) => (EventKind::ObjectMoved, *id, Some(tangible)),
        };
        Self {
            kind,
            set_id: Some(set_id),
            tangible: tangible.cloned(),
            field_set: None,
            field: None,
            count: None,
        }
    }
}

impl From<&DomainEvent> for TangibleEvent {
    fn from(event: &DomainEvent) -> Self {
        match event {
            DomainEvent::Engine(engine_event) => engine_event.into(),
            DomainEvent::Set(action) => action.into(),
        }
    }
}
