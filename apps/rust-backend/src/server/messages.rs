use serde::{Deserialize, Serialize};

// ── messages FROM client ──────────────────────────────────────────────
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Subscribe { simulation_id: u32 },
    RequestFullState { simulation_id: u32 },
}

// ── messages TO client ────────────────────────────────────────────────
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    FullState {
        simulation_id: u32,
        tick: u64,
        world_width: f32,
        world_height: f32,
        ants: Vec<AntSnapshot>,
        colonies: Vec<ColonySnapshot>,
        food_sources: Vec<FoodSnapshot>,
        pheromone_grid: PheromoneSnapshot,
    },
    DeltaUpdate {
        simulation_id: u32,
        tick: u64,
        updated_ants: Vec<AntSnapshot>,
        updated_colonies: Vec<ColonySnapshot>,
        updated_food_sources: Vec<FoodSnapshot>,
        removed_ant_ids: Vec<u32>,
        removed_food_source_ids: Vec<u32>,
    },
    Error {
        message: String,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct AntSnapshot {
    pub id: u32,
    pub position: [f32; 2],
    pub angle: f32,
    pub colony_id: u32,
    pub ant_type_id: u8,
    pub state: String,
    pub speed: f32,
    pub health: f32,
    pub energy: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ColonySnapshot {
    pub id: u32,
    pub center: [f32; 2],
    pub radius: f32,
    pub population: usize,
    pub food_stored: f32,
    pub color_hue: u16,
}

#[derive(Clone, Debug, Serialize)]
pub struct FoodSnapshot {
    pub id: u32,
    pub position: [f32; 2],
    pub food_type: String,
    pub amount: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct PheromoneSnapshot {
    pub grid_w: usize,
    pub grid_h: usize,
    pub cell_size: f32,
    pub food: Vec<u8>,
    pub home: Vec<u8>,
}
