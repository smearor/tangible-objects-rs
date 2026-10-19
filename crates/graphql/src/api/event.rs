use async_graphql::SimpleObject;
use tangible_objects_engine::EngineEvent;
use tangible_objects_model::ContactCount;
use tangible_objects_model::FieldName;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::SetAction;
use tangible_objects_model::TangibleSetId;
use tangible_objects_model::TrackedTangible;
use tangible_objects_runtime::DomainEvent;
use tangible_objects_runtime::LifecycleEvent;

use super::EventKind;

/// Flattened domain event delivered by the event subscriptions.
///
/// `kind` discriminates the payload: `tangible` carries the affected object
/// for `TANGIBLE_*`/`OBJECT_*` kinds, `setId` for `SET_*`/`OBJECT_*` kinds,
/// `field`/`fieldSet`/`count` for `FIELD_*` kinds. Lifecycle kinds use
/// `patternId`/`name`/`owner`/`detail`; `LAGGED` uses `skipped`.
#[derive(Debug, Clone, SimpleObject)]
pub struct TangibleEvent {
    /// What happened.
    pub kind: EventKind,
    /// Affected tangible for `TANGIBLE_*` and `OBJECT_*` events.
    pub tangible: Option<TrackedTangible>,
    /// Affected set for `SET_*` and `OBJECT_*` events.
    pub set_id: Option<TangibleSetId>,
    /// Field-set name for `FIELD_*` events.
    pub field_set: Option<FieldSetName>,
    /// Field name for `FIELD_*` events.
    pub field: Option<FieldName>,
    /// Debounced contact count for `FIELD_COUNT_CHANGED` events.
    pub count: Option<ContactCount>,
    /// Recorded frame count for `RECORDING_STOPPED` events.
    pub frames: Option<FrameCount>,
    /// Affected pattern for `PROFILE_*` lifecycle events.
    pub pattern_id: Option<PatternId>,
    /// Profile, field-set or stage name for lifecycle events -
    /// deliberately heterogeneous: `kind` says which.
    pub name: Option<String>,
    /// Owning application for registration events.
    pub owner: Option<Owner>,
    /// Applied value for config events (`COMPLETION_CHANGED` =
    /// `mode radius max_missing persist`, `FILTER_MODE_CHANGED` = mode).
    pub detail: Option<String>,
    /// Events dropped because the subscription lagged (`LAGGED` kind).
    pub skipped: Option<i32>,
}

impl TangibleEvent {
    /// Empty payload for `kind` - the `From` impls fill in what applies.
    fn blank(kind: EventKind) -> Self {
        Self {
            kind,
            tangible: None,
            set_id: None,
            field_set: None,
            field: None,
            count: None,
            frames: None,
            pattern_id: None,
            name: None,
            owner: None,
            detail: None,
            skipped: None,
        }
    }

    /// `LAGGED` control event: `skipped` events were dropped because this
    /// subscription lagged - the client's recovery is `snapshot` +
    /// resubscribe (SUBSCRIPTIONS.md reconnect semantics).
    pub(crate) fn lagged(skipped: u64) -> Self {
        Self {
            skipped: Some(skipped as i32),
            ..Self::blank(EventKind::Lagged)
        }
    }
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
                Some(*count),
            ),
        };
        Self {
            tangible,
            field_set,
            field,
            count,
            ..Self::blank(kind)
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
            set_id: Some(set_id),
            tangible: tangible.cloned(),
            ..Self::blank(kind)
        }
    }
}

impl From<&LifecycleEvent> for TangibleEvent {
    fn from(event: &LifecycleEvent) -> Self {
        let mut event_out = Self::blank(match event {
            LifecycleEvent::ProfileRegistered { .. } => EventKind::ProfileRegistered,
            LifecycleEvent::ProfileUnregistered { .. } => EventKind::ProfileUnregistered,
            LifecycleEvent::FieldSetRegistered { .. } => EventKind::FieldSetRegistered,
            LifecycleEvent::FieldSetUnregistered { .. } => EventKind::FieldSetUnregistered,
            LifecycleEvent::StageRegistered { .. } => EventKind::StageRegistered,
            LifecycleEvent::StageUnregistered { .. } => EventKind::StageUnregistered,
            LifecycleEvent::CompletionChanged(_) => EventKind::CompletionChanged,
            LifecycleEvent::FilterModeChanged(_) => EventKind::FilterModeChanged,
            LifecycleEvent::RecordingStarted => EventKind::RecordingStarted,
            LifecycleEvent::RecordingStopped { .. } => EventKind::RecordingStopped,
            LifecycleEvent::ConfigReloaded => EventKind::ConfigReloaded,
        });
        match event {
            LifecycleEvent::ProfileRegistered {
                pattern_id,
                name,
                owner,
            } => {
                event_out.pattern_id = Some(*pattern_id);
                event_out.name = Some(name.to_string());
                event_out.owner = owner.clone();
            }
            LifecycleEvent::ProfileUnregistered { pattern_id } => {
                event_out.pattern_id = Some(*pattern_id);
            }
            LifecycleEvent::FieldSetRegistered { name, owner } => {
                event_out.name = Some(name.to_string());
                event_out.owner = owner.clone();
            }
            LifecycleEvent::FieldSetUnregistered { name } => {
                event_out.name = Some(name.to_string());
            }
            LifecycleEvent::StageRegistered { name }
            | LifecycleEvent::StageUnregistered { name } => {
                event_out.name = Some(name.to_string());
            }
            LifecycleEvent::CompletionChanged(config) => {
                event_out.detail = Some(format!(
                    "mode={} radius={} max_missing={} persist={}",
                    config.mode, config.radius_factor, config.max_missing, config.persist
                ));
            }
            LifecycleEvent::FilterModeChanged(mode) => {
                event_out.detail = Some(mode.to_string());
            }
            LifecycleEvent::RecordingStopped { frames } => {
                event_out.frames = Some(*frames);
            }
            _ => {}
        }
        event_out
    }
}

impl From<&DomainEvent> for TangibleEvent {
    fn from(event: &DomainEvent) -> Self {
        match event {
            DomainEvent::Engine(engine_event) => engine_event.into(),
            DomainEvent::Set(action) => action.into(),
            DomainEvent::Lifecycle(lifecycle) => lifecycle.into(),
        }
    }
}
