use crate::models::*;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// High-performance in-memory cache for simulation state
/// Uses DashMap for concurrent access without full locking
#[derive(Debug, Clone)]
pub struct SimulationCache {
    // Core simulation data
    pub ants: Arc<DashMap<i32, FastAnt>>,
    pub colonies: Arc<DashMap<i32, FastColony>>,
    pub food_sources: Arc<DashMap<i32, FastFoodSource>>,
    pub pheromone_trails: Arc<DashMap<i32, FastPheromoneTrail>>,
    pub ant_types: Arc<DashMap<i32, AntType>>,
    
    // Simulation metadata
    pub simulation_id: i32,
    pub world_bounds: WorldBounds,
    pub current_tick: Arc<RwLock<i64>>,
    
    // Performance tracking
    pub last_db_sync: Arc<RwLock<i64>>,
    pub dirty_ants: Arc<DashMap<i32, bool>>,
    pub dirty_colonies: Arc<DashMap<i32, bool>>,
    pub dirty_food_sources: Arc<DashMap<i32, bool>>,
}

impl SimulationCache {
    pub fn new(simulation_id: i32, world_bounds: WorldBounds) -> Self {
        Self {
            ants: Arc::new(DashMap::new()),
            colonies: Arc::new(DashMap::new()),
            food_sources: Arc::new(DashMap::new()),
            pheromone_trails: Arc::new(DashMap::new()),
            ant_types: Arc::new(DashMap::new()),
            simulation_id,
            world_bounds,
            current_tick: Arc::new(RwLock::new(0)),
            last_db_sync: Arc::new(RwLock::new(0)),
            dirty_ants: Arc::new(DashMap::new()),
            dirty_colonies: Arc::new(DashMap::new()),
            dirty_food_sources: Arc::new(DashMap::new()),
        }
    }

    // Ant operations
    pub fn insert_ant(&self, ant: FastAnt) {
        let id = ant.id;
        self.ants.insert(id, ant);
        self.dirty_ants.insert(id, true);
    }

    pub fn update_ant<F>(&self, id: i32, updater: F)
    where
        F: FnOnce(&mut FastAnt),
    {
        if let Some(mut ant) = self.ants.get_mut(&id) {
            updater(&mut ant);
            self.dirty_ants.insert(id, true);
        }
    }

    pub fn get_ant(&self, id: &i32) -> Option<FastAnt> {
        self.ants.get(id).map(|ant| ant.clone())
    }

    pub fn remove_ant(&self, id: &i32) {
        self.ants.remove(id);
        self.dirty_ants.insert(*id, true);
    }

    pub fn get_ants_in_colony(&self, colony_id: &i32) -> Vec<FastAnt> {
        self.ants
            .iter()
            .filter(|entry| &entry.colony_id == colony_id)
            .map(|entry| entry.clone())
            .collect()
    }

    pub fn get_ants_near_position(&self, position: (f32, f32), radius: f32) -> Vec<FastAnt> {
        self.ants
            .iter()
            .filter(|entry| {
                let dx = entry.position.0 - position.0;
                let dy = entry.position.1 - position.1;
                (dx * dx + dy * dy).sqrt() <= radius
            })
            .map(|entry| entry.clone())
            .collect()
    }

    // Colony operations
    pub fn insert_colony(&self, colony: FastColony) {
        let id = colony.id;
        self.colonies.insert(id, colony);
        self.dirty_colonies.insert(id, true);
    }

    pub fn update_colony<F>(&self, id: i32, updater: F)
    where
        F: FnOnce(&mut FastColony),
    {
        if let Some(mut colony) = self.colonies.get_mut(&id) {
            updater(&mut colony);
            self.dirty_colonies.insert(id, true);
        }
    }

    pub fn get_colony(&self, id: &i32) -> Option<FastColony> {
        self.colonies.get(id).map(|colony| colony.clone())
    }

    // Food source operations
    pub fn insert_food_source(&self, food: FastFoodSource) {
        let id = food.id;
        tracing::info!("🍎 Inserting food source {}", id);
        self.food_sources.insert(id, food);
        self.dirty_food_sources.insert(id, true);
    }

    pub fn update_food_source<F>(&self, id: i32, updater: F)
    where
        F: FnOnce(&mut FastFoodSource),
    {
        if let Some(mut food) = self.food_sources.get_mut(&id) {
            tracing::info!("🍎 Updating food source {}", id);
            updater(&mut food);
            self.dirty_food_sources.insert(id, true);
        }
    }

    pub fn get_food_sources_near_position(&self, position: (f32, f32), radius: f32) -> Vec<FastFoodSource> {
        self.food_sources
            .iter()
            .filter(|entry| {
                let dx = entry.position.0 - position.0;
                let dy = entry.position.1 - position.1;
                (dx * dx + dy * dy).sqrt() <= radius
            })
            .map(|entry| entry.clone())
            .collect()
    }

    // Pheromone operations
    pub fn insert_pheromone_trail(&self, trail: FastPheromoneTrail) {
        //tracing::info!("💨 Inserting pheromone trail {}", trail.id);
        self.pheromone_trails.insert(trail.id, trail);
    }

    pub fn get_pheromone_trails_near_position(&self, position: (f32, f32), radius: f32) -> Vec<FastPheromoneTrail> {
        //tracing::info!("💨 Getting pheromone trails near {:?}", position);
        self.pheromone_trails
            .iter()
            .filter(|entry| {
                let dx = entry.position.0 - position.0;
                let dy = entry.position.1 - position.1;
                (dx * dx + dy * dy).sqrt() <= radius
            })
            .map(|entry| entry.clone())
            .collect()
    }

    // Ant type operations
    pub fn insert_ant_type(&self, ant_type: AntType) {
        self.ant_types.insert(ant_type.id, ant_type);
    }

    pub fn get_ant_type(&self, id: &i32) -> Option<AntType> {
        self.ant_types.get(id).map(|ant_type| ant_type.clone())
    }

    // Utility methods
    pub async fn get_current_tick(&self) -> i64 {
        *self.current_tick.read().await
    }

    pub async fn set_current_tick(&self, tick: i64) {
        *self.current_tick.write().await = tick;
    }

    pub async fn set_last_db_sync(&self, tick: i64) {
        *self.last_db_sync.write().await = tick;
    }

    pub fn get_dirty_ant_ids(&self) -> Vec<i32> {
        self.dirty_ants.iter().map(|entry| *entry.key()).collect()
    }

    pub fn get_dirty_colony_ids(&self) -> Vec<i32> {
        self.dirty_colonies.iter().map(|entry| *entry.key()).collect()
    }

    pub fn get_dirty_food_source_ids(&self) -> Vec<i32> {
        self.dirty_food_sources.iter().map(|entry| *entry.key()).collect()
    }

    pub fn clear_dirty_flags(&self) {
        self.dirty_ants.clear();
        self.dirty_colonies.clear();
        self.dirty_food_sources.clear();
    }

    pub fn get_stats(&self) -> SimulationStats {
        let total_ants = self.ants.len() as i32;
        let active_colonies = self.colonies.len() as i32;
        let total_food_collected = 0; // TODO: Implement proper tracking
        let pheromone_trail_count = self.pheromone_trails.len() as i32;

        SimulationStats {
            total_ants,
            active_colonies,
            total_food_collected,
            pheromone_trail_count,
            current_tick: 0, // Will be set by caller
        }
    }
} 