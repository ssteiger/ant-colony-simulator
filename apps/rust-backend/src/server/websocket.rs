use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tracing::{info, warn};

use super::messages::ClientMessage;

pub type BroadcastTx = broadcast::Sender<String>;

#[derive(Clone)]
pub struct AppState {
    pub broadcast_tx: BroadcastTx,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.broadcast_tx.subscribe();

    info!("WebSocket client connected");

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(ClientMessage::Subscribe { simulation_id }) => {
                        info!("Client subscribed to simulation {}", simulation_id);
                    }
                    Ok(ClientMessage::RequestFullState { simulation_id }) => {
                        info!(
                            "Client requested full state for simulation {}",
                            simulation_id
                        );
                    }
                    Err(e) => {
                        warn!("Failed to parse client message: {}", e);
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    info!("WebSocket client disconnected");
}
