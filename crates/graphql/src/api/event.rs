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
/// for `TANGIBLE_*`/`OBJECT_*` kinds, `setId` for `SET_*`/`OBJECT_*` kinds.
#[derive(Debug, Clone, SimpleObject)]
pub struct TangibleEvent {
    /// What happened.
    pub kind: EventKind,
    /// Affected tangible for `TANGIBLE_*` and `OBJECT_*` events.
    pub tangible: Option<TrackedTangible>,
    /// Affected set for `SET_*` and `OBJECT_*` events.
    pub set_id: Option<TangibleSetId>,
}

impl From<&EngineEvent> for TangibleEvent {
    fn from(event: &EngineEvent) -> Self {
        match event {
            EngineEvent::TouchPointsUpdated => Self {
                kind: EventKind::TouchPointsUpdated,
                tangible: None,
                set_id: None,
            },
            EngineEvent::TangibleAdded(tangible) => Self {
                kind: EventKind::TangibleAdded,
                tangible: Some(tangible.clone()),
                set_id: None,
            },
            EngineEvent::TangibleMoved(tangible) => Self {
                kind: EventKind::TangibleMoved,
                tangible: Some(tangible.clone()),
                set_id: None,
            },
            EngineEvent::TangibleRemoved(tangible) => Self {
                kind: EventKind::TangibleRemoved,
                tangible: Some(tangible.clone()),
                set_id: None,
            },
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
