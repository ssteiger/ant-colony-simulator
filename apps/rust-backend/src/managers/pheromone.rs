use crate::cache::SimulationCache;
use crate::models::{OptimizedPheromoneTrail, PheromoneType};
use std::sync::Arc;
use rand::prelude::*;
use rand::rngs::SmallRng;

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

        // Consolidate nearby trails
        self.consolidate_pheromone_trails();

        // Apply environmental effects
        self.apply_environmental_effects(current_tick);

        tracing::debug!("💨 PheromoneManager: Completed tick {}", current_tick);
    }

    fn decay_pheromone_trails(&self, current_tick: i64) {
        let mut removed_count = 0;
        let mut decayed_count = 0;

        // Collect trails that need updating
        let mut trails_to_update = Vec::new();
        
        self.cache.pheromone_trails.retain(|id, trail| {
            // Age the trail
            let age = current_tick - trail.age_ticks;
            
            // Check if trail has expired
            if current_tick >= trail.expires_at {
                removed_count += 1;
                return false; // Remove expired trails
            }

            // Apply sophisticated decay based on trail type and age
            let decay_factor = self.calculate_decay_factor(trail, age);
            let new_strength = (trail.strength * decay_factor).max(0.0);
            
            if new_strength <= 0.01 {
                removed_count += 1;
                return false; // Remove very weak trails
            }

            // Store trail for updating
            trails_to_update.push((*id, new_strength, age));
            decayed_count += 1;
            true
        });

        // Update trails outside of retain
        for (id, new_strength, age) in trails_to_update {
            if let Some(mut trail) = self.cache.pheromone_trails.get_mut(&id) {
                trail.strength = new_strength;
                trail.age_ticks = age;
            }
        }

        if removed_count > 0 || decayed_count > 0 {
            tracing::debug!("💨 Decayed {} trails, removed {} trails", decayed_count, removed_count);
        }
    }

    fn calculate_decay_factor(&self, trail: &OptimizedPheromoneTrail, age: i64) -> f32 {
        let base_decay = trail.decay_rate;
        let age_factor = (age as f32 / 100.0).min(1.0); // Normalize age
        let quality_factor = trail.quality_rating;
        
        // Higher quality trails decay slower
        let quality_modifier = 1.0 - (quality_factor * 0.3);
        
        // Consolidated trails are more stable
        let consolidation_modifier = if trail.is_consolidated { 0.7 } else { 1.0 };
        
        base_decay * age_factor * quality_modifier * consolidation_modifier
    }

    fn consolidate_pheromone_trails(&self) {
        let mut trails_to_remove = Vec::new();
        let mut trails_to_add = Vec::new();
        
        let all_trails: Vec<_> = self.cache.pheromone_trails.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        for (id1, trail1) in &all_trails {
            if trail1.is_consolidated {
                continue;
            }

            for (id2, trail2) in &all_trails {
                if id1 == id2 || trail2.is_consolidated {
                    continue;
                }

                // Check if trails are from same colony and type
                if trail1.colony_id == trail2.colony_id && trail1.trail_type == trail2.trail_type {
                    let distance = self.distance(trail1.position, trail2.position);
                    
                    // Consolidate trails that are very close together
                    if distance < 3.0 {
                        let consolidated_trail = self.merge_trails(trail1, trail2);
                        trails_to_add.push(consolidated_trail);
                        trails_to_remove.push(*id1);
                        trails_to_remove.push(*id2);
                        break;
                    }
                }
            }
        }

        // Apply changes
        for id in &trails_to_remove {
            self.cache.pheromone_trails.remove(id);
        }

        for trail in trails_to_add {
            self.cache.insert_pheromone_trail(trail);
        }

        if !trails_to_remove.is_empty() {
            tracing::debug!("💨 Consolidated {} pheromone trails", trails_to_remove.len() / 2);
        }
    }

    fn merge_trails(&self, trail1: &OptimizedPheromoneTrail, trail2: &OptimizedPheromoneTrail) -> OptimizedPheromoneTrail {
        let merged_strength = (trail1.strength + trail2.strength) * 1.2; // Bonus for merging
        let merged_quality = (trail1.quality_rating + trail2.quality_rating) / 2.0;
        let merged_reinforcement = trail1.reinforcement_count + trail2.reinforcement_count;
        
        // Calculate merged direction (weighted average)
        let direction = if let (Some(dir1), Some(dir2)) = (trail1.direction, trail2.direction) {
            let weight1 = trail1.strength / (trail1.strength + trail2.strength);
            let weight2 = trail2.strength / (trail1.strength + trail2.strength);
            Some(dir1 * weight1 + dir2 * weight2)
        } else {
            trail1.direction.or(trail2.direction)
        };

        OptimizedPheromoneTrail {
            id: trail1.id, // Keep the first trail's ID
            colony_id: trail1.colony_id,
            trail_type: trail1.trail_type,
            position: trail1.position, // Keep the first trail's position
            strength: merged_strength.min(trail1.max_strength.max(trail2.max_strength)),
            decay_rate: (trail1.decay_rate + trail2.decay_rate) / 2.0,
            expires_at: trail1.expires_at.max(trail2.expires_at),
            target_food_id: trail1.target_food_id.or(trail2.target_food_id),
            ant_id: trail1.ant_id,
            age_ticks: trail1.age_ticks.min(trail2.age_ticks), // Take the younger age
            max_strength: trail1.max_strength.max(trail2.max_strength),
            reinforcement_count: merged_reinforcement,
            quality_rating: merged_quality,
            direction,
            is_consolidated: true,
        }
    }

    fn apply_environmental_effects(&self, _current_tick: i64) {
        // Apply weather effects (if implemented)
        // Apply obstacle effects
        // Apply time-of-day effects
        
        // For now, just apply some basic environmental decay
        let environmental_decay = 0.999; // 0.1% decay per tick from environment
        
        // Collect trails that need updating
        let mut trails_to_update = Vec::new();
        
        for entry in self.cache.pheromone_trails.iter() {
            let trail = entry.value();
            if trail.strength > 0.1 {
                let new_strength = trail.strength * environmental_decay;
                trails_to_update.push((*entry.key(), new_strength));
            }
        }
        
        // Update trails
        for (id, new_strength) in trails_to_update {
            if let Some(mut trail) = self.cache.pheromone_trails.get_mut(&id) {
                trail.strength = new_strength;
            }
        }
    }

    pub fn add_pheromone_trail(&self, trail: OptimizedPheromoneTrail) {
        self.cache.insert_pheromone_trail(trail);
    }

    pub fn get_pheromone_influence(
        &self,
        position: (f32, f32),
        colony_id: i32,
        radius: f32,
        ant_role: &str,
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
                
                // Enhanced distance decay with role-specific sensitivity
                let role_sensitivity = self.get_role_sensitivity(ant_role, &trail.trail_type);
                let distance_decay = (-normalized_distance * 3.0 * role_sensitivity).exp();
                
                // Quality-based influence for food trails
                let quality_factor = if trail.trail_type == PheromoneType::Food {
                    1.0 + trail.quality_rating * 0.5
                } else {
                    1.0
                };
                
                let influence = trail.strength * distance_decay * quality_factor;

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

    fn get_role_sensitivity(&self, ant_role: &str, pheromone_type: &PheromoneType) -> f32 {
        match (ant_role, pheromone_type) {
            ("scout", PheromoneType::Exploration) => 1.5,
            ("scout", PheromoneType::Food) => 1.2,
            ("worker", PheromoneType::Food) => 1.8,
            ("worker", PheromoneType::Home) => 1.3,
            ("soldier", PheromoneType::Danger) => 2.0,
            ("soldier", PheromoneType::Enemy) => 1.8,
            ("soldier", PheromoneType::Territory) => 1.4,
            _ => 1.0,
        }
    }

    // Utility function
    fn distance(&self, pos1: (f32, f32), pos2: (f32, f32)) -> f32 {
        let dx = pos1.0 - pos2.0;
        let dy = pos1.1 - pos2.1;
        (dx * dx + dy * dy).sqrt()
    }
} 