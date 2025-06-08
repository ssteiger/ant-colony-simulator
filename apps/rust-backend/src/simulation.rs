use crate::cache::SimulationCache;
use crate::database::DatabaseManager;
use crate::managers::*;
use crate::models::*;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, Instant};

pub struct AntColonySimulator {
    cache: Arc<SimulationCache>,
    db: Arc<DatabaseManager>,
    ant_behavior_manager: AntBehaviorManager,
    colony_manager: ColonyManager,
    environment_manager: EnvironmentManager,
    pheromone_manager: PheromoneManager,
    is_running: bool,
    tick_interval: Duration,
    db_sync_interval: i64, // How often to sync to database (in ticks)
}

impl AntColonySimulator {
    pub async fn new(db_pool: sqlx::PgPool, simulation_id: i32) -> Result<Self> {
        let db = Arc::new(DatabaseManager::new(db_pool));
        
        // Load simulation data from database
        let simulation = db.load_simulation(simulation_id).await?;
        
        let world_bounds = WorldBounds {
            width: simulation.world_width as f32,
            height: simulation.world_height as f32,
        };

        let cache = Arc::new(SimulationCache::new(simulation_id, world_bounds));
        
        // Initialize cache with database data
        Self::load_initial_data(&cache, &db, simulation_id).await?;

        let ant_behavior_manager = AntBehaviorManager::new(cache.clone());
        let colony_manager = ColonyManager::new(cache.clone(), db.clone());
        let environment_manager = EnvironmentManager::new(cache.clone(), db.clone());
        let pheromone_manager = PheromoneManager::new(cache.clone());

        Ok(Self {
            cache,
            db,
            ant_behavior_manager,
            colony_manager,
            environment_manager,
            pheromone_manager,
            is_running: false,
            tick_interval: Duration::from_millis(50), // 20 FPS - much faster than Node.js
            db_sync_interval: 100, // Sync to DB every 100 ticks (5 seconds at 20 FPS)
        })
    }

    async fn load_initial_data(
        cache: &Arc<SimulationCache>,
        db: &Arc<DatabaseManager>,
        simulation_id: i32,
    ) -> Result<()> {
        tracing::info!("🚀 Loading initial simulation data...");

        // Load ant types
        let ant_types = db.load_ant_types().await?;
        for ant_type in ant_types {
            cache.insert_ant_type(ant_type);
        }
        tracing::info!("📋 Loaded {} ant types", cache.ant_types.len());

        // Load colonies
        let colonies = db.load_colonies(simulation_id).await?;
        for colony in colonies {
            let fast_colony = FastColony {
                id: colony.id,
                center: (colony.center_x as f32, colony.center_y as f32),
                radius: colony.radius as f32,
                population: colony.population,
                resources: Self::parse_resources(&colony.resources),
                territory_radius: colony.territory_radius as f32,
                aggression_level: colony.aggression_level as f32,
            };
            cache.insert_colony(fast_colony);
        }
        tracing::info!("🏰 Loaded {} colonies", cache.colonies.len());

        // Load ants
        let ants = db.load_ants(simulation_id).await?;
        for ant in ants {
            let fast_ant = FastAnt {
                id: ant.id,
                colony_id: ant.colony_id,
                ant_type_id: ant.ant_type_id,
                position: (ant.position_x as f32, ant.position_y as f32),
                angle: ant.angle as f32,
                speed: ant.current_speed as f32,
                health: ant.health,
                energy: ant.energy,
                age_ticks: ant.age_ticks,
                state: Self::parse_ant_state(&ant.state),
                target: Self::parse_ant_target(&ant.target_type, ant.target_id, ant.target_x, ant.target_y),
                carried_resources: Self::parse_carried_resources(&ant.carried_resources),
                last_action_tick: 0,
            };
            cache.insert_ant(fast_ant);
        }
        tracing::info!("🐜 Loaded {} ants", cache.ants.len());

        // Load food sources
        let food_sources = db.load_food_sources(simulation_id).await?;
        for food in food_sources {
            let fast_food = FastFoodSource {
                id: food.id,
                position: (food.position_x as f32, food.position_y as f32),
                food_type: food.food_type,
                amount: food.amount,
                max_amount: food.max_amount,
                regeneration_rate: food.regeneration_rate.unwrap_or(0) as f32 / 100.0, // Convert to decimal
                is_renewable: food.is_renewable.unwrap_or(false),
            };
            cache.insert_food_source(fast_food);
        }
        tracing::info!("🍎 Loaded {} food sources", cache.food_sources.len());

        tracing::info!("✅ Initial data loading complete");
        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.is_running {
            return Err(anyhow::anyhow!("Simulation is already running"));
        }

        tracing::info!("🎮 Starting ant colony simulation...");
        self.is_running = true;

        let mut ticker = interval(self.tick_interval);
        let mut current_tick = self.cache.get_current_tick().await;

        loop {
            if !self.is_running {
                break;
            }

            ticker.tick().await;
            let tick_start = Instant::now();

            current_tick += 1;
            self.cache.set_current_tick(current_tick).await;

            // Process simulation tick
            if let Err(e) = self.process_tick(current_tick).await {
                tracing::error!("Error processing tick {}: {}", current_tick, e);
            }

            // Periodic database synchronization (much less frequent than Node.js)
            if current_tick % self.db_sync_interval == 0 {
                if let Err(e) = self.sync_to_database(current_tick).await {
                    tracing::error!("Failed to sync to database at tick {}: {}", current_tick, e);
                }
            }

            let tick_duration = tick_start.elapsed();
            
            // Log progress every 100 ticks
            if current_tick % 100 == 0 {
                let stats = self.cache.get_stats();
                tracing::info!(
                    "📊 Tick {} - Ants: {}, Colonies: {}, Food: {}, Pheromones: {} ({}ms)",
                    current_tick,
                    stats.total_ants,
                    stats.active_colonies,
                    stats.total_food_collected,
                    stats.pheromone_trail_count,
                    tick_duration.as_millis()
                );
            }

            // Warn if tick is taking too long
            if tick_duration > self.tick_interval {
                tracing::warn!(
                    "Tick {} took {}ms (target: {}ms)",
                    current_tick,
                    tick_duration.as_millis(),
                    self.tick_interval.as_millis()
                );
            }
        }

        // Final sync to database when stopping
        self.sync_to_database(current_tick).await?;
        tracing::info!("🛑 Simulation stopped at tick {}", current_tick);
        Ok(())
    }

    pub async fn stop(&mut self) {
        tracing::info!("🛑 Stopping simulation...");
        self.is_running = false;
    }

    async fn process_tick(&self, current_tick: i64) -> Result<()> {
        // Process all managers in optimal order
        
        // 1. Environment updates (food regeneration, weather)
        self.environment_manager.process_tick(current_tick).await;

        // 2. Pheromone trail decay
        self.pheromone_manager.process_tick(current_tick).await;

        // 3. Ant behavior (the heaviest computation)
        self.ant_behavior_manager.process_tick(current_tick).await;

        // 4. Colony management (spawning, resource consumption)
        self.colony_manager.process_tick(current_tick).await;

        Ok(())
    }

    async fn sync_to_database(&self, current_tick: i64) -> Result<()> {
        let sync_start = Instant::now();

        // Get dirty entities
        let dirty_ant_ids = self.cache.get_dirty_ant_ids();
        let dirty_colony_ids = self.cache.get_dirty_colony_ids();
        let dirty_food_ids = self.cache.get_dirty_food_source_ids();

        // Batch update ants
        if !dirty_ant_ids.is_empty() {
            let dirty_ants: Vec<FastAnt> = dirty_ant_ids
                .iter()
                .filter_map(|id| self.cache.get_ant(id))
                .collect();

            if !dirty_ants.is_empty() {
                self.db.batch_update_ants(&dirty_ants).await?;
            }
        }

        // Batch update colonies
        if !dirty_colony_ids.is_empty() {
            let dirty_colonies: Vec<FastColony> = dirty_colony_ids
                .iter()
                .filter_map(|id| self.cache.get_colony(id))
                .collect();

            if !dirty_colonies.is_empty() {
                self.db.batch_update_colonies(&dirty_colonies).await?;
            }
        }

        // Batch update food sources
        if !dirty_food_ids.is_empty() {
            let dirty_food_sources: Vec<FastFoodSource> = dirty_food_ids
                .iter()
                .filter_map(|id| {
                    self.cache.food_sources.get(id).map(|entry| entry.clone())
                })
                .collect();

            if !dirty_food_sources.is_empty() {
                self.db.batch_update_food_sources(&dirty_food_sources).await?;
            }
        }

        // Update simulation tick
        self.db.update_simulation_tick(self.cache.simulation_id, current_tick).await?;

        self.cache.clear_dirty_flags();
        self.cache.set_last_db_sync(current_tick).await;

        let sync_duration = sync_start.elapsed();
        tracing::debug!(
            "💾 Database sync complete - Updated {} ants, {} colonies, {} food sources ({}ms)",
            dirty_ant_ids.len(),
            dirty_colony_ids.len(),
            dirty_food_ids.len(),
            sync_duration.as_millis()
        );

        Ok(())
    }

    pub fn get_stats(&self) -> SimulationStats {
        let mut stats = self.cache.get_stats();
        stats.current_tick = self.cache.current_tick.try_read().map(|t| *t).unwrap_or(0);
        stats
    }

    // Utility functions for data conversion
    fn parse_resources(resources_json: &serde_json::Value) -> std::collections::HashMap<String, i32> {
        let mut resources = std::collections::HashMap::new();
        
        if let Some(obj) = resources_json.as_object() {
            for (key, value) in obj {
                if let Some(amount) = value.as_i64() {
                    resources.insert(key.clone(), amount as i32);
                }
            }
        }
        
        resources
    }

    fn parse_ant_state(state_str: &str) -> AntState {
        match state_str {
            "wandering" => AntState::Wandering,
            "seeking_food" => AntState::SeekingFood,
            "carrying_food" => AntState::CarryingFood,
            "returning_to_colony" => AntState::ReturningToColony,
            "following" => AntState::Following,
            "exploring" => AntState::Exploring,
            "patrolling" => AntState::Patrolling,
            "dead" => AntState::Dead,
            _ => AntState::Wandering,
        }
    }

    fn parse_ant_target(
        target_type: &Option<String>,
        target_id: Option<i32>,
        target_x: Option<i32>,
        target_y: Option<i32>,
    ) -> Option<Target> {
        match target_type.as_deref() {
            Some("food_source") => target_id.map(Target::Food),
            Some("colony") => target_id.map(Target::Colony),
            Some("position") => {
                if let (Some(x), Some(y)) = (target_x, target_y) {
                    Some(Target::Position(x as f32, y as f32))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_carried_resources(
        carried_json: &Option<serde_json::Value>,
    ) -> std::collections::HashMap<String, i32> {
        let mut resources = std::collections::HashMap::new();
        
        if let Some(json) = carried_json {
            if let Some(obj) = json.as_object() {
                for (key, value) in obj {
                    if let Some(amount) = value.as_i64() {
                        resources.insert(key.clone(), amount as i32);
                    }
                }
            }
        }
        
        resources
    }
} 