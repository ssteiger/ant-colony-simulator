use crate::cache::SimulationCache;
use crate::models::*;
use std::sync::Arc;

pub struct PheromoneManager {
    cache: Arc<SimulationCache>,
}

impl PheromoneManager {
    pub fn new(cache: Arc<SimulationCache>) -> Self {
        Self { cache }
    }

    pub async fn process_tick(&self, current_tick: i64) {
        tracing::debug!("💨 PheromoneManager: Processing tick {}", current_tick);

        // Decay and remove expired pheromone trails
        self.decay_pheromone_trails(current_tick);

        tracing::debug!("💨 PheromoneManager: Completed tick {}", current_tick);
    }

    fn decay_pheromone_trails(&self, current_tick: i64) {
        let mut removed_count = 0;
        let mut decayed_count = 0;

        self.cache.pheromone_trails.retain(|_, trail| {
            // Check if trail has expired
            if current_tick >= trail.expires_at {
                removed_count += 1;
                return false; // Remove expired trails
            }

            // Apply decay
            let new_strength = (trail.strength - trail.decay_rate).max(0.0);
            if new_strength <= 0.01 {
                removed_count += 1;
                return false; // Remove very weak trails
            }

            // Update strength (this is safe because we're inside retain)
            unsafe {
                let trail_ptr = trail as *const FastPheromoneTrail as *mut FastPheromoneTrail;
                (*trail_ptr).strength = new_strength;
                decayed_count += 1;
            }

            true
        });

        if removed_count > 0 || decayed_count > 0 {
            tracing::debug!("💨 Decayed {} trails, removed {} trails", decayed_count, removed_count);
        }
    }

    pub fn add_pheromone_trail(&self, trail: FastPheromoneTrail) {
        self.cache.insert_pheromone_trail(trail);
    }

    pub fn get_pheromone_influence(
        &self,
        position: (f32, f32),
        colony_id: i32,
        radius: f32,
    ) -> (f32, f32) {
        let radius_sq = radius * radius;
        let mut total_influence_x = 0.0;
        let mut total_influence_y = 0.0;
        let mut total_strength = 0.0;

        for entry in self.cache.pheromone_trails.iter() {
            let trail = entry.value();
            if trail.colony_id != colony_id || trail.strength <= 0.1 {
                continue;
            }

            let dx = trail.position.0 - position.0;
            let dy = trail.position.1 - position.1;
            let distance_sq = dx * dx + dy * dy;

            if distance_sq <= radius_sq && distance_sq > 0.0 {
                let distance = distance_sq.sqrt();
                let normalized_distance = distance / radius;
                let distance_decay = (-normalized_distance * 3.0).exp();
                let influence = trail.strength * distance_decay;

                let dir_x = dx / distance;
                let dir_y = dy / distance;

                total_influence_x += dir_x * influence;
                total_influence_y += dir_y * influence;
                total_strength += influence;
            }
        }

        if total_strength == 0.0 {
            (0.0, 0.0)
        } else {
            (
                total_influence_x.atan2(total_influence_y),
                total_strength,
            )
        }
    }
} 