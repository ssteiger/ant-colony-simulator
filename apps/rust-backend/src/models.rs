use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Simulation {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub world_width: i32,
    pub world_height: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub simulation_speed: Option<i32>,
    pub current_tick: Option<i64>,
    pub season: Option<String>,
    pub time_of_day: Option<i32>,
    pub weather_type: Option<String>,
    pub weather_intensity: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Colony {
    pub id: i32,
    pub simulation_id: i32,
    pub name: String,
    pub center_x: i32,
    pub center_y: i32,
    pub radius: i32,
    pub population: i32,
    pub color_hue: i32,
    pub resources: serde_json::Value,
    pub nest_level: i32,
    pub territory_radius: i32,
    pub aggression_level: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ant {
    pub id: i32,
    pub colony_id: i32,
    pub ant_type_id: i32,
    pub position_x: i32,
    pub position_y: i32,
    pub angle: i32,
    pub current_speed: i32,
    pub health: i32,
    pub age_ticks: i32,
    pub state: String,
    pub target_x: Option<i32>,
    pub target_y: Option<i32>,
    pub target_type: Option<String>,
    pub target_id: Option<i32>,
    pub carried_resources: Option<serde_json::Value>,
    pub traits: Option<serde_json::Value>,
    pub energy: i32,
    pub mood: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntType {
    pub id: i32,
    pub name: String,
    pub base_speed: i32,
    pub base_strength: i32,
    pub base_health: i32,
    pub base_size: i32,
    pub lifespan_ticks: i32,
    pub carrying_capacity: i32,
    pub role: String,
    pub color_hue: i32,
    pub special_abilities: Option<serde_json::Value>,
    pub food_preferences: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodSource {
    pub id: i32,
    pub simulation_id: i32,
    pub food_type: String,
    pub position_x: i32,
    pub position_y: i32,
    pub amount: i32,
    pub max_amount: i32,
    pub regeneration_rate: Option<i32>,
    pub discovery_difficulty: Option<i32>,
    pub nutritional_value: i32,
    pub spoilage_rate: Option<i32>,
    pub is_renewable: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PheromoneTrail {
    pub id: i32,
    pub colony_id: i32,
    pub trail_type: String,
    pub position_x: i32,
    pub position_y: i32,
    pub strength: i32,
    pub decay_rate: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub source_ant_id: Option<i32>,
    pub target_food_id: Option<i32>,
    pub ant_id: i32,
}

// Runtime optimized structures for in-memory processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastAnt {
    pub id: i32,
    pub colony_id: i32,
    pub ant_type_id: i32,
    pub position: (f32, f32),
    pub angle: f32,
    pub speed: f32,
    pub health: i32,
    pub energy: i32,
    pub age_ticks: i32,
    pub state: AntState,
    pub target: Option<Target>,
    pub carried_resources: HashMap<String, i32>,
    pub last_action_tick: i64,
    pub last_food_source_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastColony {
    pub id: i32,
    pub center: (f32, f32),
    pub radius: f32,
    pub population: i32,
    pub resources: HashMap<String, i32>,
    pub territory_radius: f32,
    pub aggression_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastFoodSource {
    pub id: i32,
    pub position: (f32, f32),
    pub food_type: String,
    pub amount: i32,
    pub max_amount: i32,
    pub regeneration_rate: f32,
    pub is_renewable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPheromoneTrail {
    pub id: i32,
    pub colony_id: i32,
    pub trail_type: PheromoneType,
    pub position: (f32, f32),
    pub strength: f32,
    pub decay_rate: f32,
    pub expires_at: i64, // Tick when it expires
    pub target_food_id: Option<i32>,
    pub ant_id: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AntState {
    Wandering,
    SeekingFood,
    CarryingFood,
    Following,
    Exploring,
    Patrolling,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Target {
    Food(i32),
    Colony(i32),
    Position(f32, f32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PheromoneType {
    Food,
    Danger,
    Home,
    Exploration,
}

#[derive(Debug, Clone)]
pub struct WorldBounds {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct SimulationStats {
    pub total_ants: i32,
    pub active_colonies: i32,
    pub total_food_collected: i32,
    pub pheromone_trail_count: i32,
    pub current_tick: i64,
} 