use serde::Deserialize;

/// JSON messages FROM the client.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Subscribe { simulation_id: i32 },
}

/// Control messages forwarded from WebSocket handlers to the simulation thread.
#[derive(Debug)]
pub enum ControlMsg {
    Subscribe { simulation_id: i32 },
}
