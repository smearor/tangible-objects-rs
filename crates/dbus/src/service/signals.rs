//! Domain-event to D-Bus signal mapping - `emit_signal` dispatches
//! one [`DomainEvent`] onto the matching `EngineService` signal.

use tangible_objects_engine::EngineEvent;
use tangible_objects_model::SetAction;
use tangible_objects_runtime::DomainEvent;
use tracing::warn;
use zbus::object_server::SignalEmitter;

use crate::service::engine_service::EngineService;

/// Maps one domain event onto the matching D-Bus signal.
pub(crate) async fn emit_signal(emitter: &SignalEmitter<'_>, event: &DomainEvent) {
    let result = match event {
        DomainEvent::Engine(EngineEvent::TouchPointsUpdated) => {
            EngineService::touch_points_updated(emitter).await
        }
        DomainEvent::Engine(EngineEvent::TangibleAdded(tangible)) => {
            EngineService::tangible_added(emitter, tangible).await
        }
        DomainEvent::Engine(EngineEvent::TangibleMoved(tangible)) => {
            EngineService::tangible_moved(emitter, tangible).await
        }
        DomainEvent::Engine(EngineEvent::TangibleRemoved(tangible)) => {
            EngineService::tangible_removed(emitter, tangible).await
        }
        DomainEvent::Engine(EngineEvent::FieldPlaced { set, field }) => {
            EngineService::field_placed(emitter, set.as_str(), field.as_str()).await
        }
        DomainEvent::Engine(EngineEvent::FieldLifted { set, field }) => {
            EngineService::field_lifted(emitter, set.as_str(), field.as_str()).await
        }
        DomainEvent::Engine(EngineEvent::FieldCountChanged { set, field, count }) => {
            EngineService::field_count_changed(
                emitter,
                set.as_str(),
                field.as_str(),
                count.as_u32(),
            )
            .await
        }
        DomainEvent::Set(action) => emit_set_action(emitter, action).await,
        // Lifecycle events flow only to GraphQL subscriptions - no D-Bus
        // signal counterparts (SUBSCRIPTIONS.md scope).
        DomainEvent::Lifecycle(_) => Ok(()),
    };
    if let Err(err) = result {
        warn!(%err, "failed to emit D-Bus signal");
    }
}

/// Emits the signal for a Tangible Object Set action.
async fn emit_set_action(emitter: &SignalEmitter<'_>, action: &SetAction) -> zbus::Result<()> {
    match action {
        SetAction::SetAdded(set_id) => EngineService::set_added(emitter, *set_id).await,
        SetAction::SetRemoved(set_id) => EngineService::set_removed(emitter, *set_id).await,
        SetAction::ObjectAdded(set_id, tangible) => {
            EngineService::object_added(emitter, *set_id, tangible).await
        }
        SetAction::ObjectRemoved(set_id, tangible) => {
            EngineService::object_removed(emitter, *set_id, tangible).await
        }
        SetAction::ObjectMoved(set_id, tangible) => {
            EngineService::object_moved(emitter, *set_id, tangible).await
        }
    }
}
