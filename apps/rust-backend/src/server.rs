use crate::websocket::WebSocketManager;
use axum::{
    routing::get,
    Router,
    http::Method,
};
use tower_http::cors::{CorsLayer, Any};
use tokio::net::TcpListener;
use tracing::info;

pub struct SimulationServer {
    websocket_manager: WebSocketManager,
}

impl SimulationServer {
    pub fn new(websocket_manager: WebSocketManager) -> Self {
        Self {
            websocket_manager,
        }
    }

    pub async fn start(&self, addr: &str) -> anyhow::Result<()> {
        let app = self.create_router();
        
        info!("🚀 Starting simulation server on {}", addr);
        let listener = TcpListener::bind(addr).await?;
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }

    fn create_router(&self) -> Router {
        Router::new()
            .route("/ws", get(WebSocketManager::handle_websocket))
            .route("/health", get(health_check))
            .route("/stats", get(get_server_stats))
            .with_state(self.websocket_manager.clone())
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods([Method::GET, Method::POST])
                    .allow_headers(Any),
            )
    }
}

async fn health_check() -> &'static str {
    "OK"
}

async fn get_server_stats(
    axum::extract::State(websocket_manager): axum::extract::State<WebSocketManager>,
) -> String {
    let client_count = websocket_manager.client_count().await;
    format!("{{\"connected_clients\": {}}}", client_count)
} 