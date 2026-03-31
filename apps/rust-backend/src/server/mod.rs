pub mod messages;
pub mod websocket;

use std::sync::Arc;

use axum::http::Method;
use axum::routing::get;
use axum::Router;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use websocket::{ws_handler, AppState, BroadcastTx};

pub fn create_broadcast() -> BroadcastTx {
    let (tx, _) = broadcast::channel::<String>(64);
    tx
}

pub async fn start_server(addr: &str, broadcast_tx: BroadcastTx) -> anyhow::Result<()> {
    let state = Arc::new(AppState { broadcast_tx });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(|| async { "OK" }))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET])
                .allow_headers(Any),
        )
        .with_state(state);

    info!("WebSocket server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
