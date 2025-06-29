use bevy::prelude::*;
use big_brain::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// CORE COMPONENTS
// ============================================================================

/// Marker component for ants
#[derive(Component, Debug, Clone)]
pub struct Ant;

/// Marker component for colonies
#[derive(Component, Debug, Clone)]
pub struct Colony;

/// Marker component for food sources
#[derive(Component, Debug, Clone)]
pub struct FoodSource;

/// Marker component for pheromone trails
#[derive(Component, Debug, Clone)]
pub struct PheromoneTrail;

// ============================================================================
// ANT COMPONENTS
// ============================================================================

/// Physical properties of an ant
#[derive(Component, Debug, Clone)]
pub struct AntPhysics {
    pub position: Vec2,
    pub velocity: Vec2,
    pub max_speed: f32,
    pub acceleration: f32,
    pub rotation: f32,
    pub rotation_speed: f32,
}

/// Health and energy system for ants
#[derive(Component, Debug, Clone)]
pub struct AntHealth {
    pub health: f32,
    pub max_health: f32,
    pub energy: f32,
    pub max_energy: f32,
    pub age_ticks: i64,
    pub lifespan_ticks: i64,
}

/// Ant state and behavior
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AntState {
    Wandering,
    SeekingFood,
    CarryingFood,
    Following,
    Exploring,
    Patrolling,
    Dead,
}

/// Resources carried by an ant
#[derive(Component, Debug, Clone, Default)]
pub struct CarriedResources {
    pub resources: HashMap<String, f32>,
    pub capacity: f32,
    pub current_weight: f32,
}

/// Ant target system
#[derive(Component, Debug, Clone)]
pub enum AntTarget {
    Food(Entity),
    Colony(Entity),
    Position(Vec2),
    None,
}

/// Ant memory and knowledge
#[derive(Component, Debug, Clone)]
pub struct AntMemory {
    pub known_food_sources: Vec<Entity>,
    pub known_colonies: Vec<Entity>,
    pub last_food_source: Option<Entity>,
    pub last_action_tick: i64,
    pub pheromone_sensitivity: f32,
}

/// Ant type and role
#[derive(Component, Debug, Clone)]
pub struct AntType {
    pub name: String,
    pub role: String,
    pub base_speed: f32,
    pub base_strength: f32,
    pub base_health: f32,
    pub carrying_capacity: f32,
    pub color_hue: f32,
    pub special_abilities: Vec<String>,
}

// ============================================================================
// COLONY COMPONENTS
// ============================================================================

/// Colony properties
#[derive(Component, Debug, Clone)]
pub struct ColonyProperties {
    pub name: String,
    pub center: Vec2,
    pub radius: f32,
    pub population: i32,
    pub max_population: i32,
    pub color_hue: f32,
    pub territory_radius: f32,
    pub aggression_level: f32,
}

/// Colony resources
#[derive(Component, Debug, Clone)]
pub struct ColonyResources {
    pub resources: HashMap<String, f32>,
    pub storage_capacity: HashMap<String, f32>,
}

/// Colony nest level and upgrades
#[derive(Component, Debug, Clone)]
pub struct ColonyNest {
    pub level: i32,
    pub max_level: i32,
    pub upgrade_cost: HashMap<String, f32>,
}

// ============================================================================
// FOOD SOURCE COMPONENTS
// ============================================================================

/// Food source properties
#[derive(Component, Debug, Clone)]
pub struct FoodSourceProperties {
    pub food_type: String,
    pub amount: f32,
    pub max_amount: f32,
    pub regeneration_rate: f32,
    pub is_renewable: bool,
    pub nutritional_value: f32,
    pub spoilage_rate: f32,
    pub discovery_difficulty: f32,
}

// ============================================================================
// PHEROMONE COMPONENTS
// ============================================================================

/// Pheromone trail properties
#[derive(Component, Debug, Clone)]
pub struct PheromoneProperties {
    pub trail_type: PheromoneType,
    pub strength: f32,
    pub max_strength: f32,
    pub decay_rate: f32,
    pub expires_at: i64,
    pub source_ant: Option<Entity>,
    pub target_food: Option<Entity>,
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PheromoneType {
    Food,
    Danger,
    Home,
    Exploration,
}

// ============================================================================
// SIMULATION COMPONENTS
// ============================================================================

/// Simulation state
#[derive(Resource, Debug, Clone)]
pub struct SimulationState {
    pub simulation_id: i32,
    pub current_tick: i64,
    pub world_bounds: WorldBounds,
    pub is_running: bool,
    pub simulation_speed: f32,
}

/// World boundaries
#[derive(Resource, Debug, Clone)]
pub struct WorldBounds {
    pub width: f32,
    pub height: f32,
}

/// Simulation statistics
#[derive(Resource, Debug, Clone)]
pub struct SimulationStats {
    pub total_ants: i32,
    pub active_colonies: i32,
    pub total_food_collected: f32,
    pub pheromone_trail_count: i32,
    pub current_tick: i64,
}

// ============================================================================
// BIG BRAIN AI COMPONENTS
// ============================================================================

/// Ant AI brain using Big Brain
#[derive(Component, Debug)]
pub struct AntBrain {
    pub thinker: Thinker,
}

/// Scorer for food seeking behavior
#[derive(Component, Debug, Clone)]
pub struct FoodSeekingScorer {
    pub hunger_threshold: f32,
    pub food_memory_weight: f32,
}

/// Scorer for pheromone following behavior
#[derive(Component, Debug, Clone)]
pub struct PheromoneFollowingScorer {
    pub pheromone_strength_threshold: f32,
    pub colony_loyalty: f32,
}

/// Scorer for exploration behavior
#[derive(Component, Debug, Clone)]
pub struct ExplorationScorer {
    pub exploration_urge: f32,
    pub boredom_threshold: f32,
}

/// Scorer for returning to colony
#[derive(Component, Debug, Clone)]
pub struct ReturnToColonyScorer {
    pub energy_threshold: f32,
    pub resource_weight_threshold: f32,
}

// ============================================================================
// DATABASE MODELS (for persistence)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSimulation {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub world_width: i32,
    pub world_height: i32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: Option<bool>,
    pub simulation_speed: Option<i32>,
    pub current_tick: Option<i64>,
    pub season: Option<String>,
    pub time_of_day: Option<i32>,
    pub weather_type: Option<String>,
    pub weather_intensity: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseColony {
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
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseAnt {
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
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseAntType {
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
pub struct DatabaseFoodSource {
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
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePheromoneTrail {
    pub id: i32,
    pub colony_id: i32,
    pub trail_type: String,
    pub position_x: i32,
    pub position_y: i32,
    pub strength: i32,
    pub decay_rate: i32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub source_ant_id: Option<i32>,
    pub target_food_id: Option<i32>,
    pub ant_id: i32,
}
