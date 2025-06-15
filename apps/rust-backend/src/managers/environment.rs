use crate::cache::SimulationCache;
use crate::database::DatabaseManager;
use crate::models::*;
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::sync::Arc;

pub struct EnvironmentManager {
    cache: Arc<SimulationCache>,
    db: Arc<DatabaseManager>,
}

impl EnvironmentManager {
    pub fn new(cache: Arc<SimulationCache>, db: Arc<DatabaseManager>) -> Self {
        Self { cache, db }
    }

    pub async fn process_tick(&self, current_tick: i64) {
        tracing::debug!("🌍 EnvironmentManager: Processing tick {}", current_tick);

        // Regenerate food sources
        self.regenerate_food_sources();

        // Spawn new food sources every 100 ticks
        if current_tick % 100 == 0 {
            if let Err(e) = self.spawn_new_food_sources().await {
                tracing::warn!("Failed to spawn new food sources: {}", e);
            }
        }

        tracing::debug!("🌍 EnvironmentManager: Completed tick {}", current_tick);
    }

    fn regenerate_food_sources(&self) {
        let mut regenerated_count = 0;
        
        for mut entry in self.cache.food_sources.iter_mut() {
            let food = entry.value_mut();
            
            // Only regenerate every 10 ticks and if food is renewable
            if food.is_renewable && food.amount < food.max_amount && food.regeneration_rate > 0.0 {
                // Reduce regeneration rate by 10x
                let regeneration_amount = food.regeneration_rate / 10.0;
                let new_amount = (food.amount as f32 + regeneration_amount).min(food.max_amount as f32) as i32;
                
                if new_amount > food.amount {
                    food.amount = new_amount;
                    regenerated_count += 1;
                    tracing::info!("🍎 Food source {} regenerated to {}", entry.key(), new_amount);
                }
            }
        }

        if regenerated_count > 0 {
            tracing::info!("🌍 Regenerated {} food sources", regenerated_count);
        }
    }

    async fn spawn_new_food_sources(&self) -> anyhow::Result<()> {
        let current_food_count = self.cache.food_sources.len();
        let max_food_sources = 20; // Limit total food sources

        if current_food_count >= max_food_sources {
            return Ok(());
        }

        let mut rng = SmallRng::from_entropy();
        let food_types = vec!["seeds", "sugar", "protein", "fruit"];
        
        // Spawn 1-3 new food sources
        let spawn_count = rng.gen_range(1..=3);
        
        for _ in 0..spawn_count {
            if self.cache.food_sources.len() >= max_food_sources {
                break;
            }

            let food_type = food_types.choose(&mut rng).unwrap().to_string();
            let position_x = rng.gen_range(50.0..self.cache.world_bounds.width - 50.0) as i32;
            let position_y = rng.gen_range(50.0..self.cache.world_bounds.height - 50.0) as i32;
            
            let amount = rng.gen_range(20..=50);
            let max_amount = amount + rng.gen_range(10..=30);
            let regeneration_rate = if rng.gen_bool(0.3) { rng.gen_range(1..=3) } else { 0 };
            let is_renewable = regeneration_rate > 0;

            let new_food = FastFoodSource {
                id: rng.gen::<i32>().abs(), // Temporary ID - should be replaced by database
                position: (position_x as f32, position_y as f32),
                food_type,
                amount,
                max_amount,
                regeneration_rate: regeneration_rate as f32,
                is_renewable,
            };

            self.cache.insert_food_source(new_food);
            
            tracing::debug!("🍎 Spawned new food source at ({}, {})", position_x, position_y);
        }

        Ok(())
    }

    pub fn apply_weather_effects(&self, _weather_type: &str, _intensity: f32) {
        // Placeholder for weather effects
        // Could affect food regeneration, ant movement speed, etc.
    }

    pub fn handle_seasonal_changes(&self, _season: &str) {
        // Placeholder for seasonal effects
        // Could affect food availability, ant behavior, etc.
    }
} 