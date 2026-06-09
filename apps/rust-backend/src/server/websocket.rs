use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tracing::{info, warn};

use super::messages::{ClientMessage, ControlMsg};

/// Outbound frame, shared cheaply between all connected clients.
#[derive(Clone)]
pub enum WsOut {
    Binary(Arc<Vec<u8>>),
    Text(Arc<String>),
}

pub type BroadcastTx = broadcast::Sender<WsOut>;
pub type ControlTx = std::sync::mpsc::Sender<ControlMsg>;

pub struct AppState {
    pub broadcast_tx: BroadcastTx,
    pub control_tx: ControlTx,
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
    let control_tx = state.control_tx.clone();

    info!("WebSocket client connected");

    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(WsOut::Binary(bytes)) => {
                    if sender
                        .send(Message::Binary(bytes.as_ref().clone()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Ok(WsOut::Text(text)) => {
                    if sender
                        .send(Message::Text(text.as_ref().clone()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                // client too slow for the broadcast buffer: skip missed frames
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("WebSocket client lagged, skipped {} frames", n);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(ClientMessage::Subscribe { simulation_id }) => {
                        info!("Client subscribed to simulation {}", simulation_id);
                        let _ = control_tx.send(ControlMsg::Subscribe { simulation_id });
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
