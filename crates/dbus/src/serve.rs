//! `serve`/`spawn` - runs the D-Bus service on the session bus and
//! forwards domain events as signals until the device loop stops.

use crate::service::EngineService;
use crate::service::OwnerSessions;
use crate::service::signals::emit_signal;
use std::future::pending;
use std::thread::JoinHandle;
use tangible_objects_model::Owner;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::EngineHandle;
use tokio::runtime::Runtime;
use tokio::sync::broadcast::error::RecvError;
use tracing::error;
use tracing::info;
use tracing::warn;
use zbus::connection;
use zbus::fdo::DBusProxy;
use zbus::object_server::SignalEmitter;

/// Well-known bus name of the daemon's D-Bus service.
pub const BUS_NAME: &str = "org.tangibleobjects.Engine";

/// Object path of the engine service.
pub const OBJECT_PATH: &str = "/org/tangibleobjects/Engine";

/// Serves the D-Bus interface on the session bus and forwards domain
/// events as signals until the device loop stops.
pub async fn serve(handle: EngineHandle) -> zbus::Result<()> {
    let owner_sessions: OwnerSessions = Default::default();
    let connection = connection::Builder::session()?
        .name(BUS_NAME)?
        .serve_at(
            OBJECT_PATH,
            EngineService::new(handle.clone(), owner_sessions.clone()),
        )?
        .build()
        .await?;
    info!(
        bus = BUS_NAME,
        path = OBJECT_PATH,
        "D-Bus service listening"
    );

    // Disconnect cleanup: when a caller's unique bus name vanishes,
    // unregister everything that name had registered (session-scoped).
    let proxy = DBusProxy::new(&connection).await?;
    let mut owner_changes = proxy.receive_name_owner_changed().await?;
    let cleanup_handle = handle.clone();
    tokio::spawn(async move {
        use futures_lite::StreamExt;

        while let Some(signal) = owner_changes.next().await {
            let Ok(args) = signal.args() else {
                continue;
            };
            if args.new_owner().is_some() {
                continue;
            }
            let name = args.name().to_string();
            let owners: Vec<Owner> = owner_sessions
                .lock()
                .map(|mut sessions| {
                    sessions
                        .remove(&name)
                        .unwrap_or_default()
                        .into_iter()
                        .collect()
                })
                .unwrap_or_default();
            for owner in owners {
                info!(%owner, %name, "D-Bus caller vanished - unregistering");
                let _ = cleanup_handle
                    .send_command(ControlCommand::UnregisterOwner { owner })
                    .await;
            }
        }
    });

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
pub fn spawn(handle: EngineHandle) -> JoinHandle<()> {
    std::thread::spawn(move || match Runtime::new() {
        Ok(runtime) => {
            if let Err(err) = runtime.block_on(serve(handle)) {
                error!(%err, "D-Bus service failed");
            }
        }
        Err(err) => error!(%err, "failed to start D-Bus runtime"),
    })
}
