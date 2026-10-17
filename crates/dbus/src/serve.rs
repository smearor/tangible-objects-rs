use std::future::pending;

use tangible_objects_engine::EngineEvent;
use tangible_objects_model::SetAction;
use tangible_objects_runtime::DomainEvent;
use tangible_objects_runtime::EngineHandle;
use tokio::sync::broadcast::error::RecvError;
use tracing::error;
use tracing::info;
use tracing::warn;
use zbus::connection;
use zbus::object_server::SignalEmitter;

use super::EngineService;

/// Well-known bus name of the daemon's D-Bus service.
pub const BUS_NAME: &str = "org.tangibleobjects.Engine";

/// Object path of the engine service.
pub const OBJECT_PATH: &str = "/org/tangibleobjects/Engine";

/// Serves the D-Bus interface on the session bus and forwards domain
/// events as signals until the device loop stops.
pub async fn serve(handle: EngineHandle) -> zbus::Result<()> {
    let connection = connection::Builder::session()?
        .name(BUS_NAME)?
        .serve_at(OBJECT_PATH, EngineService::new(handle.clone()))?
        .build()
        .await?;
    info!(
        bus = BUS_NAME,
        path = OBJECT_PATH,
        "D-Bus service listening"
    );

    let emitter = SignalEmitter::new(&connection, OBJECT_PATH)?;
    let mut events = handle.subscribe();
    loop {
        match events.recv().await {
            Ok(event) => emit_signal(&emitter, &event).await,
            Err(RecvError::Lagged(skipped)) => {
                warn!(skipped, "D-Bus signal forwarder lagged");
            }
            Err(RecvError::Closed) => {
                info!("event stream closed, stopping D-Bus signals");
                break;
            }
        }
    }

    // Keep the connection alive even after the event stream ends so
    // methods stay callable.
    pending::<()>().await;
    Ok(())
}

/// Spawns the D-Bus service on a dedicated thread with its own tokio
/// runtime, keeping the daemon's synchronous device loop untouched.
pub fn spawn(handle: EngineHandle) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || match tokio::runtime::Runtime::new() {
        Ok(runtime) => {
            if let Err(err) = runtime.block_on(serve(handle)) {
                error!(%err, "D-Bus service failed");
            }
        }
        Err(err) => error!(%err, "failed to start D-Bus runtime"),
    })
}

/// Maps one domain event onto the matching D-Bus signal.
async fn emit_signal(emitter: &SignalEmitter<'_>, event: &DomainEvent) {
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
            EngineService::field_placed(emitter, set, field).await
        }
        DomainEvent::Engine(EngineEvent::FieldLifted { set, field }) => {
            EngineService::field_lifted(emitter, set, field).await
        }
        DomainEvent::Engine(EngineEvent::FieldCountChanged { set, field, count }) => {
            EngineService::field_count_changed(emitter, set, field, count.as_u32()).await
        }
        DomainEvent::Set(action) => emit_set_action(emitter, action).await,
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
