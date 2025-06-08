use crate::models::*;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn};
use futures_util::{SinkExt, StreamExt};

/// WebSocket message types for real-time simulation updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SimulationMessage {
    /// Full simulation state (sent on initial connection)
    FullState {
        simulation_id: i32,
        tick: i64,
        ants: Vec<FastAnt>,
        colonies: Vec<FastColony>,
        food_sources: Vec<FastFoodSource>,
        pheromone_trails: Vec<FastPheromoneTrail>,
    },
    /// Delta update with only changed entities
    DeltaUpdate {
        simulation_id: i32,
        tick: i64,
        updated_ants: Vec<FastAnt>,
        updated_colonies: Vec<FastColony>,
        updated_food_sources: Vec<FastFoodSource>,
        new_pheromone_trails: Vec<FastPheromoneTrail>,
        removed_ant_ids: Vec<i32>,
        removed_food_source_ids: Vec<i32>,
    },
    /// Simulation control messages
    SimulationStatus {
        simulation_id: i32,
        is_running: bool,
        current_tick: i64,
    },
    /// Error messages
    Error { message: String },
}

/// Manages WebSocket connections and broadcasts simulation updates
#[derive(Clone)]
pub struct WebSocketManager {
    /// Broadcast channel for sending updates to all connected clients
    tx: broadcast::Sender<SimulationMessage>,
    /// Track connected clients count
    connected_clients: Arc<RwLock<u32>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1000); // Buffer up to 1000 messages
        
        Self {
            tx,
            connected_clients: Arc::new(RwLock::new(0)),
        }
    }

    /// Get the number of connected clients
    pub async fn client_count(&self) -> u32 {
        *self.connected_clients.read().await
    }

    /// Broadcast a simulation message to all connected clients
    pub fn broadcast(&self, message: SimulationMessage) -> Result<usize, broadcast::error::SendError<SimulationMessage>> {
        self.tx.send(message)
    }

    /// Handle WebSocket upgrade request
    pub async fn handle_websocket(
        ws: WebSocketUpgrade,
        State(manager): State<WebSocketManager>,
    ) -> Response {
        ws.on_upgrade(move |socket| async move {
            if let Err(e) = manager.handle_socket(socket).await {
                error!("WebSocket connection error: {}", e);
            }
        })
    }

    /// Handle individual WebSocket connection
    async fn handle_socket(&self, socket: WebSocket) -> anyhow::Result<()> {
        let mut rx = self.tx.subscribe();
        
        // Increment client count
        {
            let mut count = self.connected_clients.write().await;
            *count += 1;
            info!("New WebSocket client connected. Total clients: {}", *count);
        }

        let (mut sender, mut receiver) = socket.split();

        // Handle incoming messages from client (if needed for control)
        let handle_incoming = tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        info!("Received WebSocket message: {}", text);
                        // Handle client messages (e.g., subscription to specific simulation)
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket client disconnected");
                        break;
                    }
                    Err(e) => {
                        warn!("WebSocket receive error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Handle outgoing messages to client
        let handle_outgoing = tokio::spawn(async move {
            while let Ok(message) = rx.recv().await {
                let json = match serde_json::to_string(&message) {
                    Ok(json) => json,
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };

                if sender.send(Message::Text(json)).await.is_err() {
                    warn!("Failed to send WebSocket message, client likely disconnected");
                    break;
                }
            }
        });

        // Wait for either task to complete (client disconnect or error)
        tokio::select! {
            _ = handle_incoming => {},
            _ = handle_outgoing => {},
        }

        // Decrement client count
        {
            let mut count = self.connected_clients.write().await;
            *count = count.saturating_sub(1);
            info!("WebSocket client disconnected. Total clients: {}", *count);
        }

        Ok(())
    }
}

/// Create delta update by comparing current state with previous state
pub fn create_delta_update(
    simulation_id: i32,
    current_tick: i64,
    current_ants: &[FastAnt],
    current_colonies: &[FastColony],
    current_food_sources: &[FastFoodSource],
    current_pheromone_trails: &[FastPheromoneTrail],
    previous_ants: &[FastAnt],
    previous_colonies: &[FastColony],
    previous_food_sources: &[FastFoodSource],
) -> SimulationMessage {
    // Find updated ants (position, state, or other properties changed)
    let updated_ants: Vec<FastAnt> = current_ants
        .iter()
        .filter(|current_ant| {
            previous_ants
                .iter()
                .find(|prev_ant| prev_ant.id == current_ant.id)
                .map_or(true, |prev_ant| {
                    // Check if ant has changed significantly
                    prev_ant.position != current_ant.position
                        || prev_ant.state != current_ant.state
                        || prev_ant.health != current_ant.health
                        || prev_ant.energy != current_ant.energy
                })
        })
        .cloned()
        .collect();

    // Find updated colonies (resources, population changed)
    let updated_colonies: Vec<FastColony> = current_colonies
        .iter()
        .filter(|current_colony| {
            previous_colonies
                .iter()
                .find(|prev_colony| prev_colony.id == current_colony.id)
                .map_or(true, |prev_colony| {
                    prev_colony.population != current_colony.population
                        || prev_colony.resources != current_colony.resources
                })
        })
        .cloned()
        .collect();

    // Find updated food sources (amount changed)
    let updated_food_sources: Vec<FastFoodSource> = current_food_sources
        .iter()
        .filter(|current_food| {
            previous_food_sources
                .iter()
                .find(|prev_food| prev_food.id == current_food.id)
                .map_or(true, |prev_food| prev_food.amount != current_food.amount)
        })
        .cloned()
        .collect();

    // New pheromone trails (assume all current trails are new for simplicity)
    let new_pheromone_trails = current_pheromone_trails.to_vec();

    // Find removed entities
    let removed_ant_ids: Vec<i32> = previous_ants
        .iter()
        .filter(|prev_ant| !current_ants.iter().any(|curr_ant| curr_ant.id == prev_ant.id))
        .map(|ant| ant.id)
        .collect();

    let removed_food_source_ids: Vec<i32> = previous_food_sources
        .iter()
        .filter(|prev_food| !current_food_sources.iter().any(|curr_food| curr_food.id == prev_food.id))
        .map(|food| food.id)
        .collect();

    SimulationMessage::DeltaUpdate {
        simulation_id,
        tick: current_tick,
        updated_ants,
        updated_colonies,
        updated_food_sources,
        new_pheromone_trails,
        removed_ant_ids,
        removed_food_source_ids,
    }
} 