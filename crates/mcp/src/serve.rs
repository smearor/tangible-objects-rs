use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use tangible_objects_runtime::EngineHandle;
use tracing::error;
use tracing::info;

use super::EngineMcp;

/// Serves MCP on stdin/stdout until the client disconnects.
///
/// Only usable when the daemon does not write other output to stdout.
pub async fn serve_stdio(handle: EngineHandle) -> io::Result<()> {
    let service = EngineMcp::new(handle)
        .serve(stdio())
        .await
        .map_err(io::Error::other)?;
    service.waiting().await.map_err(io::Error::other)?;
    Ok(())
}

/// Serves the MCP streamable-HTTP endpoint on `/mcp`.
pub async fn serve_http(handle: EngineHandle, addr: SocketAddr) -> io::Result<()> {
    let service = StreamableHttpService::new(
        move || Ok(EngineMcp::new(handle.clone())),
        Arc::new(LocalSessionManager::default()),
        Default::default(),
    );
    let app = Router::new().route_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "MCP endpoint listening");
    axum::serve(listener, app).await
}

/// Spawns the MCP HTTP endpoint on a dedicated thread with its own tokio
/// runtime, keeping the daemon's synchronous device loop untouched.
pub fn spawn_http(handle: EngineHandle, addr: SocketAddr) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || match tokio::runtime::Runtime::new() {
        Ok(runtime) => {
            if let Err(err) = runtime.block_on(serve_http(handle, addr)) {
                error!(%err, "MCP server failed");
            }
        }
        Err(err) => error!(%err, "failed to start MCP runtime"),
    })
}
