use axum::{
    routing::get,
    Router,
    http::Method,
};
use tower_http::cors::{CorsLayer, Any};
use tokio::net::TcpListener;
use tracing::info;

pub struct SimulationServer;

impl SimulationServer {
    pub fn new() -> Self {
        Self
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
            .route("/health", get(health_check))
            .route("/stats", get(get_server_stats))
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

async fn get_server_stats() -> String {
    format!("{{\"connected_clients\": 0}}")
} 