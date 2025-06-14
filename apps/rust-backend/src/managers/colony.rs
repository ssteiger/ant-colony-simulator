use crate::cache::SimulationCache;
use crate::database::DatabaseManager;
use crate::models::*;
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ColonyManager {
    cache: Arc<SimulationCache>,
    db: Arc<DatabaseManager>,
}

impl ColonyManager {
    pub fn new(cache: Arc<SimulationCache>, db: Arc<DatabaseManager>) -> Self {
        Self { cache, db }
    }

    pub async fn process_tick(&self, current_tick: i64) {
        tracing::debug!("🏰 ColonyManager: Processing tick {}", current_tick);

        let colony_ids: Vec<i32> = self.cache.colonies.iter()
            .map(|entry| *entry.key())
            .collect();

        if colony_ids.is_empty() {
            tracing::warn!("🏰 No colonies found - attempting to create initial colonies");
            if let Err(e) = self.create_initial_colonies().await {
                tracing::error!("Failed to create initial colonies: {}", e);
            }
            return;
        }

        for colony_id in colony_ids {
            if let Err(e) = self.process_colony(colony_id, current_tick).await {
                tracing::warn!("Failed to process colony {}: {}", colony_id, e);
            }
        }

        tracing::debug!("🏰 ColonyManager: Completed tick {}", current_tick);
    }

    async fn process_colony(&self, colony_id: i32, current_tick: i64) -> anyhow::Result<()> {
        let colony = match self.cache.get_colony(&colony_id) {
            Some(colony) => colony,
            None => return Ok(()), // Colony doesn't exist
        };

        // Update population count based on living ants
        let living_ants = self.cache.get_ants_in_colony(&colony_id);
        let new_population = living_ants.len() as i32;

        if new_population != colony.population {
            self.cache.update_colony(colony_id, |col| {
                col.population = new_population;
            });
        }

        /*
        // Spawn new ants if colony is below minimum population
        if new_population < 50 && current_tick % 20 == 0 {
            if let Err(e) = self.spawn_ant(colony_id, current_tick).await {
                tracing::warn!("Failed to spawn ant for colony {}: {}", colony_id, e);
            }
        }
        */

        // Consume resources
        self.consume_colony_resources(colony_id, new_population);

        Ok(())
    }

    async fn spawn_ant(&self, colony_id: i32, _current_tick: i64) -> anyhow::Result<()> {
        let colony = match self.cache.get_colony(&colony_id) {
            Some(colony) => colony,
            None => return Ok(()),
        };

        // Get a worker ant type (assume ID 1 exists)
        let ant_type_id = 1;

        // Spawn position near colony center with some randomness
        let mut rng = SmallRng::from_entropy();
        let spawn_offset = 10.0;
        let spawn_x = colony.center.0 + rng.gen_range(-spawn_offset..spawn_offset);
        let spawn_y = colony.center.1 + rng.gen_range(-spawn_offset..spawn_offset);

        // Ensure spawn position is within world bounds
        let spawn_x = spawn_x.clamp(0.0, self.cache.world_bounds.width);
        let spawn_y = spawn_y.clamp(0.0, self.cache.world_bounds.height);

        // Create new ant
        let new_ant_id = self.db.spawn_ant(colony_id, ant_type_id, (spawn_x, spawn_y)).await?;

        // Add to cache
        let fast_ant = FastAnt {
            id: new_ant_id,
            colony_id,
            ant_type_id,
            position: (spawn_x, spawn_y),
            angle: rng.gen_range(0.0..2.0 * std::f32::consts::PI),
            speed: 1.0,
            health: 100,
            energy: 100,
            age_ticks: 0,
            state: AntState::Wandering,
            target: None,
            carried_resources: HashMap::new(),
            last_action_tick: 0,
        };

        self.cache.insert_ant(fast_ant);

        tracing::debug!("🐜 Spawned new ant {} for colony {}", new_ant_id, colony_id);
        Ok(())
    }

    fn consume_colony_resources(&self, colony_id: i32, population: i32) {
        if population == 0 {
            return;
        }

        // Simple resource consumption: each ant consumes 0.1 food per tick
        let food_consumption = (population as f32 * 0.1) as i32;

        self.cache.update_colony(colony_id, |colony| {
            // Consume food resources, starting with seeds
            let mut remaining_consumption = food_consumption;

            if let Some(seeds) = colony.resources.get_mut("seeds") {
                let consumed = remaining_consumption.min(*seeds);
                *seeds -= consumed;
                remaining_consumption -= consumed;
            }

            if remaining_consumption > 0 {
                if let Some(sugar) = colony.resources.get_mut("sugar") {
                    let consumed = remaining_consumption.min(*sugar);
                    *sugar -= consumed;
                    remaining_consumption -= consumed;
                }
            }

            if remaining_consumption > 0 {
                if let Some(protein) = colony.resources.get_mut("protein") {
                    let consumed = remaining_consumption.min(*protein);
                    *protein -= consumed;
                }
            }
        });
    }

    fn calculate_spawn_rate(&self, colony: &FastColony, current_population: i32) -> f32 {
        // Base spawn rate depends on available resources
        let total_resources: i32 = colony.resources.values().sum();
        let resource_factor = (total_resources as f32 / 100.0).min(2.0); // Cap at 2x

        // Population factor - spawn more when population is low
        let population_factor = if current_population < 20 {
            2.0
        } else if current_population < 50 {
            1.0
        } else {
            0.5
        };

        // Territory factor - larger territory allows more ants
        let territory_factor = (colony.territory_radius / 100.0).clamp(0.5, 2.0);

        resource_factor * population_factor * territory_factor
    }

    async fn create_initial_colonies(&self) -> anyhow::Result<()> {
        tracing::info!("🏰 Creating initial colonies...");

        let simulation_id = self.cache.simulation_id;
        let world_bounds = &self.cache.world_bounds;

        // Create colonies in database first
        let db_colonies = self.db.create_initial_colonies(
            simulation_id,
            world_bounds.width as i32,
            world_bounds.height as i32,
        ).await?;

        // Add colonies to cache
        for db_colony in db_colonies {
            let fast_colony = FastColony {
                id: db_colony.id,
                center: (db_colony.center_x as f32, db_colony.center_y as f32),
                radius: db_colony.radius as f32,
                population: db_colony.population,
                resources: Self::parse_resources(&db_colony.resources),
                territory_radius: db_colony.territory_radius as f32,
                aggression_level: db_colony.aggression_level as f32,
            };

            self.cache.insert_colony(fast_colony);
        }

        tracing::info!("🏰 Successfully created {} initial colonies", self.cache.colonies.len());
        Ok(())
    }

    fn parse_resources(resources_json: &serde_json::Value) -> HashMap<String, i32> {
        let mut resources = HashMap::new();
        
        if let Some(obj) = resources_json.as_object() {
            for (key, value) in obj {
                if let Some(amount) = value.as_i64() {
                    resources.insert(key.clone(), amount as i32);
                }
            }
        }
        
        resources
    }

    pub fn get_colony_stats(&self, colony_id: &i32) -> Option<ColonyStats> {
        let colony = self.cache.get_colony(colony_id)?;
        let living_ants = self.cache.get_ants_in_colony(colony_id);
        
        let average_health = if !living_ants.is_empty() {
            living_ants.iter().map(|ant| ant.health as f32).sum::<f32>() / living_ants.len() as f32
        } else {
            0.0
        };

        let average_energy = if !living_ants.is_empty() {
            living_ants.iter().map(|ant| ant.energy as f32).sum::<f32>() / living_ants.len() as f32
        } else {
            0.0
        };

        let total_resources: i32 = colony.resources.values().sum();

        Some(ColonyStats {
            colony_id: *colony_id,
            population: living_ants.len() as i32,
            total_resources,
            average_health,
            average_energy,
            territory_radius: colony.territory_radius,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ColonyStats {
    pub colony_id: i32,
    pub population: i32,
    pub total_resources: i32,
    pub average_health: f32,
    pub average_energy: f32,
    pub territory_radius: f32,
} 