use crate::cache::SimulationCache;
use crate::models::{OptimizedAnt, OptimizedPheromoneTrail, AntState, PheromoneType, Target};
use crate::managers::pheromone::PheromoneManager;
use rand::prelude::*;
use rand::rngs::SmallRng;
use rayon::prelude::*;
use std::sync::Arc;

pub struct AntBehaviorManager {
    cache: Arc<SimulationCache>,
}

impl AntBehaviorManager {
    // Movement constraints as associated constants
    const MAX_TURN_RATE: f32 = 0.1; // Maximum radians per tick

    pub fn new(cache: Arc<SimulationCache>) -> Self {
        Self { cache }
    }

    pub async fn process_tick(&self, current_tick: i64) {
        tracing::debug!("🐜 AntBehaviorManager: Processing tick {}", current_tick);

        // Get all living ants efficiently
        let living_ants: Vec<_> = self.cache.ants
            .iter()
            .filter(|entry| entry.state != AntState::Dead)
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        if living_ants.is_empty() {
            tracing::debug!("🐜 No living ants to process");
            return;
        }

        tracing::debug!("🐜 Processing {} living ants", living_ants.len());

        // Process ants in parallel batches for optimal performance
        const BATCH_SIZE: usize = 100;
        living_ants
            .par_chunks(BATCH_SIZE)
            .for_each(|batch| {
                for (ant_id, ant) in batch {
                    if let Err(e) = self.process_ant_behavior(&ant, current_tick) {
                        tracing::warn!("Failed to process ant {}: {}", ant_id, e);
                    }
                }
            });

        tracing::debug!("🐜 Completed tick {} processing", current_tick);
    }

    fn process_ant_behavior(&self, ant: &OptimizedAnt, current_tick: i64) -> anyhow::Result<()> {
        // Skip dead ants
        if ant.state == AntState::Dead {
            return Ok(());
        }

        // Determine what action the ant should take
        let action = self.determine_ant_action(ant)?;
        
        // Execute the action
        self.execute_ant_action(ant, action, current_tick)?;

        Ok(())
    }

    fn determine_ant_action(&self, ant: &OptimizedAnt) -> anyhow::Result<AntAction> {
        match ant.state {
            AntState::Wandering => {
                // Random chance to start exploring or seeking food
                let mut rng = SmallRng::from_entropy();
                if rng.gen::<f32>() < 0.1 {
                    Ok(AntAction::Explore)
                } else {
                    Ok(AntAction::Wander)
                }
            }
            AntState::SeekingFood => {
                // Look for nearby food sources
                let nearby_food = self.cache.get_food_sources_near_position(ant.position, 5.0);
                if !nearby_food.is_empty() {
                    Ok(AntAction::MoveToFood(nearby_food[0].id))
                } else {
                    Ok(AntAction::FollowPheromone)
                }
            }
            AntState::CarryingFood => {
                // Return to colony with food
                Ok(AntAction::ReturnToColony)
            }
            AntState::Following => {
                // Continue following pheromone trail
                Ok(AntAction::FollowPheromone)
            }
            AntState::Exploring => {
                // Continue exploring
                Ok(AntAction::Explore)
            }
            AntState::Patrolling => {
                // Continue patrolling
                Ok(AntAction::Patrol)
            }
            AntState::Dead => {
                Ok(AntAction::None)
            }
        }
    }

    fn execute_ant_action(&self, ant: &OptimizedAnt, action: AntAction, current_tick: i64) -> anyhow::Result<()> {
        match action {
            AntAction::Wander => self.move_ant_randomly(ant)?,
            AntAction::Explore => self.scout_explore(ant)?,
            AntAction::Patrol => self.soldier_patrol(ant)?,
            AntAction::MoveToFood(food_id) => self.move_ant_towards_food(ant, food_id)?,
            AntAction::FollowPheromone => self.follow_pheromone_trail(ant, 0.0, 1.0)?,
            AntAction::MoveToTarget => self.move_ant_towards_target(ant)?,
            AntAction::CollectFood(food_id) => self.collect_food(ant, food_id, current_tick)?,
            AntAction::ReturnToColony => self.move_ant_towards_colony(ant, current_tick)?,
            AntAction::DepositFood => self.deposit_food(ant, current_tick)?,
            AntAction::None => {}
        }
        Ok(())
    }

    fn move_ant_randomly(&self, ant: &OptimizedAnt) -> anyhow::Result<()> {
        let mut rng = SmallRng::from_entropy();
        
        // Random direction change
        let angle_change = rng.gen_range(-0.5..0.5);
        let new_angle = ant.angle + angle_change;
        
        // Move in the new direction
        let distance = ant.speed;
        let new_x = ant.position.0 + new_angle.cos() * distance;
        let new_y = ant.position.1 + new_angle.sin() * distance;
        
        // Update ant position
        self.cache.update_ant(ant.id, |ant| {
            ant.position = (new_x, new_y);
            ant.angle = new_angle;
        });
        
        Ok(())
    }

    fn scout_explore(&self, ant: &OptimizedAnt) -> anyhow::Result<()> {
        let mut rng = SmallRng::from_entropy();
        
        // Scouts move more systematically
        let exploration_angle = ant.angle + rng.gen_range(-0.3..0.3);
        let distance = ant.speed * 1.5; // Scouts move faster
        
        let new_x = ant.position.0 + exploration_angle.cos() * distance;
        let new_y = ant.position.1 + exploration_angle.sin() * distance;
        
        // Update ant position
        self.cache.update_ant(ant.id, |ant| {
            ant.position = (new_x, new_y);
            ant.angle = exploration_angle;
        });
        
        // Drop exploration pheromone
        self.drop_exploration_pheromone(ant)?;
        
        Ok(())
    }

    fn soldier_patrol(&self, ant: &OptimizedAnt) -> anyhow::Result<()> {
        let mut rng = SmallRng::from_entropy();
        
        // Soldiers patrol in a more structured pattern
        let patrol_angle = ant.angle + rng.gen_range(-0.2..0.2);
        let distance = ant.speed * 0.8; // Soldiers move slower but more deliberately
        
        let new_x = ant.position.0 + patrol_angle.cos() * distance;
        let new_y = ant.position.1 + patrol_angle.sin() * distance;
        
        // Update ant position
        self.cache.update_ant(ant.id, |ant| {
            ant.position = (new_x, new_y);
            ant.angle = patrol_angle;
        });
        
        // Drop territory pheromone
        self.drop_territory_pheromone(ant)?;
        
        Ok(())
    }

    fn move_ant_towards_food(&self, ant: &OptimizedAnt, food_id: i32) -> anyhow::Result<()> {
        // Get food source position
        if let Some(food) = self.cache.get_food_source(&food_id) {
            let dx = food.position.0 - ant.position.0;
            let dy = food.position.1 - ant.position.1;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance < 2.0 {
                // Close enough to collect food
                self.cache.update_ant(ant.id, |ant| {
                    ant.state = AntState::CarryingFood;
                    ant.target = Some(Target::Food(food_id));
                });
            } else {
                // Move towards food
                let angle = dy.atan2(dx);
                let new_x = ant.position.0 + angle.cos() * ant.speed;
                let new_y = ant.position.1 + angle.sin() * ant.speed;
                
                self.cache.update_ant(ant.id, |ant| {
                    ant.position = (new_x, new_y);
                    ant.angle = angle;
                });
            }
        }
        
        Ok(())
    }

    fn follow_pheromone_trail(&self, ant: &OptimizedAnt, direction: f32, strength: f32) -> anyhow::Result<()> {
        // Get nearby pheromone trails
        let nearby_trails = self.cache.get_pheromone_trails_near_position(ant.position, 10.0);
        
        if !nearby_trails.is_empty() {
            // Find the strongest trail
            let strongest_trail = nearby_trails.iter()
                .filter(|trail| trail.colony_id == ant.colony_id)
                .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(&nearby_trails[0]);
            
            // Move towards the trail
            let dx = strongest_trail.position.0 - ant.position.0;
            let dy = strongest_trail.position.1 - ant.position.1;
            let angle = dy.atan2(dx);
            
            let new_x = ant.position.0 + angle.cos() * ant.speed;
            let new_y = ant.position.1 + angle.sin() * ant.speed;
            
            self.cache.update_ant(ant.id, |ant| {
                ant.position = (new_x, new_y);
                ant.angle = angle;
                ant.state = AntState::Following;
            });
        } else {
            // No pheromone trail found, start wandering
            self.cache.update_ant(ant.id, |ant| {
                ant.state = AntState::Wandering;
            });
        }
        
        Ok(())
    }

    fn move_ant_towards_target(&self, ant: &OptimizedAnt) -> anyhow::Result<()> {
        if let Some(target) = &ant.target {
            match target {
                Target::Food(food_id) => {
                    self.move_ant_towards_food(ant, *food_id)?;
                }
                Target::Colony(colony_id) => {
                    if let Some(colony) = self.cache.get_colony(colony_id) {
                        let dx = colony.center.0 - ant.position.0;
                        let dy = colony.center.1 - ant.position.1;
                        let angle = dy.atan2(dx);
                        
                        let new_x = ant.position.0 + angle.cos() * ant.speed;
                        let new_y = ant.position.1 + angle.sin() * ant.speed;
                        
                        self.cache.update_ant(ant.id, |ant| {
                            ant.position = (new_x, new_y);
                            ant.angle = angle;
                        });
                    }
                }
                Target::Position(x, y) => {
                    let dx = x - ant.position.0;
                    let dy = y - ant.position.1;
                    let angle = dy.atan2(dx);
                    
                    let new_x = ant.position.0 + angle.cos() * ant.speed;
                    let new_y = ant.position.1 + angle.sin() * ant.speed;
                    
                    self.cache.update_ant(ant.id, |ant| {
                        ant.position = (new_x, new_y);
                        ant.angle = angle;
                    });
                }
            }
        }
        
        Ok(())
    }

    fn collect_food(&self, ant: &OptimizedAnt, food_id: i32, _current_tick: i64) -> anyhow::Result<()> {
        if let Some(mut food) = self.cache.food_sources.get_mut(&food_id) {
            let amount_to_collect = 1.min(food.amount);
            food.amount -= amount_to_collect;
            
            // Update ant's carried resources
            self.cache.update_ant(ant.id, |ant| {
                *ant.carried_resources.entry("food".to_string()).or_insert(0) += amount_to_collect;
                ant.state = AntState::CarryingFood;
                ant.target = Some(Target::Colony(ant.colony_id));
            });
            
            // Drop food pheromone trail
            self.drop_food_pheromone(ant, food_id)?;
        }
        
        Ok(())
    }

    fn move_ant_towards_colony(&self, ant: &OptimizedAnt, current_tick: i64) -> anyhow::Result<()> {
        if let Some(colony) = self.cache.get_colony(ant.colony_id) {
            let dx = colony.center.0 - ant.position.0;
            let dy = colony.center.1 - ant.position.1;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance < colony.radius {
                // Reached colony, deposit food
                self.cache.update_ant(ant.id, |ant| {
                    ant.state = AntState::Wandering;
                    ant.target = None;
                });
            } else {
                // Move towards colony
                let angle = dy.atan2(dx);
                let new_x = ant.position.0 + angle.cos() * ant.speed;
                let new_y = ant.position.1 + angle.sin() * ant.speed;
                
                self.cache.update_ant(ant.id, |ant| {
                    ant.position = (new_x, new_y);
                    ant.angle = angle;
                });
                
                // Drop home pheromone trail
                self.drop_home_pheromone(ant)?;
            }
        }
        
        Ok(())
    }

    fn deposit_food(&self, ant: &OptimizedAnt, _current_tick: i64) -> anyhow::Result<()> {
        if let Some(colony) = self.cache.get_colony(ant.colony_id) {
            let food_amount = ant.carried_resources.get("food").unwrap_or(&0);
            
            // Update colony resources
            self.cache.update_colony(ant.colony_id, |colony| {
                *colony.resources.entry("food".to_string()).or_insert(0) += food_amount;
            });
            
            // Clear ant's carried resources
            self.cache.update_ant(ant.id, |ant| {
                ant.carried_resources.clear();
                ant.state = AntState::Wandering;
                ant.target = None;
            });
        }
        
        Ok(())
    }

    fn drop_exploration_pheromone(&self, ant: &OptimizedAnt) -> anyhow::Result<()> {
        let mut rng = SmallRng::from_entropy();
        let current_tick = self.cache.current_tick.try_read().map(|t| *t).unwrap_or(0);
        
        let trail = OptimizedPheromoneTrail {
            id: rng.gen::<i32>().abs(),
            colony_id: ant.colony_id,
            trail_type: PheromoneType::Exploration,
            position: ant.position,
            strength: 0.1,
            decay_rate: 0.0006,
            expires_at: current_tick + 8000,
            target_food_id: None,
            ant_id: ant.id,
            age_ticks: current_tick,
            max_strength: 0.2,
            reinforcement_count: 0,
            quality_rating: 1.0,
            direction: Some(ant.angle),
            is_consolidated: false,
        };

        self.cache.insert_pheromone_trail(trail);
        Ok(())
    }

    fn drop_territory_pheromone(&self, ant: &OptimizedAnt) -> anyhow::Result<()> {
        let mut rng = SmallRng::from_entropy();
        let current_tick = self.cache.current_tick.try_read().map(|t| *t).unwrap_or(0);
        
        let trail = OptimizedPheromoneTrail {
            id: rng.gen::<i32>().abs(),
            colony_id: ant.colony_id,
            trail_type: PheromoneType::Territory,
            position: ant.position,
            strength: 0.5,
            decay_rate: 0.00005,
            expires_at: current_tick + 50000,
            target_food_id: None,
            ant_id: ant.id,
            age_ticks: current_tick,
            max_strength: 1.0,
            reinforcement_count: 0,
            quality_rating: 1.0,
            direction: None,
            is_consolidated: false,
        };

        self.cache.insert_pheromone_trail(trail);
        Ok(())
    }

    fn drop_food_pheromone(&self, ant: &OptimizedAnt, food_id: i32) -> anyhow::Result<()> {
        let mut rng = SmallRng::from_entropy();
        let current_tick = self.cache.current_tick.try_read().map(|t| *t).unwrap_or(0);
        
        let trail = OptimizedPheromoneTrail {
            id: rng.gen::<i32>().abs(),
            colony_id: ant.colony_id,
            trail_type: PheromoneType::Food,
            position: ant.position,
            strength: 0.8,
            decay_rate: 0.0003,
            expires_at: current_tick + 15000,
            target_food_id: Some(food_id),
            ant_id: ant.id,
            age_ticks: current_tick,
            max_strength: 1.6,
            reinforcement_count: 0,
            quality_rating: 1.0,
            direction: Some(ant.angle),
            is_consolidated: false,
        };

        self.cache.insert_pheromone_trail(trail);
        Ok(())
    }

    fn drop_home_pheromone(&self, ant: &OptimizedAnt) -> anyhow::Result<()> {
        let mut rng = SmallRng::from_entropy();
        let current_tick = self.cache.current_tick.try_read().map(|t| *t).unwrap_or(0);
        
        let trail = OptimizedPheromoneTrail {
            id: rng.gen::<i32>().abs(),
            colony_id: ant.colony_id,
            trail_type: PheromoneType::Home,
            position: ant.position,
            strength: 0.6,
            decay_rate: 0.0004,
            expires_at: current_tick + 12000,
            target_food_id: None,
            ant_id: ant.id,
            age_ticks: current_tick,
            max_strength: 1.2,
            reinforcement_count: 0,
            quality_rating: 1.0,
            direction: None,
            is_consolidated: false,
        };

        self.cache.insert_pheromone_trail(trail);
        Ok(())
    }

    fn kill_ant(&self, ant_id: i32, _current_tick: i64) {
        self.cache.update_ant(ant_id, |ant| {
            ant.state = AntState::Dead;
            ant.health = 0;
        });
        tracing::debug!("💀 Ant {} has died", ant_id);
    }

    fn create_pheromone_trail(&self, position: (f32, f32), colony_id: i32, trail_type: PheromoneType, strength: f32, target_food_id: Option<i32>, ant_id: i32) {
        let mut rng = SmallRng::from_entropy();
        let current_tick = self.cache.current_tick.try_read().map(|t| *t).unwrap_or(0);
        
        // Get ant type for role-specific pheromone behavior
        let ant_type = self.cache.get_ant_type(&self.cache.ants.get(&ant_id).map(|ant| ant.ant_type_id).unwrap_or(0));
        let role = ant_type.as_ref().map(|at| at.role.as_str()).unwrap_or("worker");
        
        // Calculate quality rating for food trails
        let quality_rating = if trail_type == PheromoneType::Food {
            if let Some(food_id) = target_food_id {
                if let Some(food) = self.cache.food_sources.get(&food_id) {
                    // Quality based on food amount and type
                    let amount_factor = (food.amount as f32 / food.max_amount as f32).min(1.0);
                    let type_factor = match food.food_type.as_str() {
                        "protein" => 1.0,
                        "sugar" => 0.8,
                        "fruit" => 0.9,
                        _ => 0.7,
                    };
                    amount_factor * type_factor
                } else {
                    0.5
                }
            } else {
                0.5
            }
        } else {
            1.0
        };

        // Role-specific pheromone strength adjustments
        let role_strength_multiplier = match role {
            "scout" => 0.8,  // Scouts create weaker trails
            "worker" => 1.2, // Workers create stronger trails
            "soldier" => 0.6, // Soldiers create weak trails
            _ => 1.0,
        };

        // Calculate direction for directional pheromones
        let direction = if let Some(ant) = self.cache.ants.get(&ant_id) {
            Some(ant.angle)
        } else {
            None
        };

        let trail = OptimizedPheromoneTrail {
            id: rng.gen::<i32>().abs(),
            colony_id,
            trail_type,
            position,
            strength: strength * role_strength_multiplier,
            decay_rate: self.get_decay_rate_for_type(&trail_type, role),
            expires_at: current_tick + self.get_expiration_for_type(&trail_type),
            target_food_id,
            ant_id,
            age_ticks: current_tick,
            max_strength: strength * 2.0, // Can be reinforced up to 2x original strength
            reinforcement_count: 0,
            quality_rating,
            direction,
            is_consolidated: false,
        };

        self.cache.insert_pheromone_trail(trail);
    }

    fn get_decay_rate_for_type(&self, trail_type: &PheromoneType, role: &str) -> f32 {
        let base_decay = match trail_type {
            PheromoneType::Food => 0.0003,
            PheromoneType::Danger => 0.0001,
            PheromoneType::Home => 0.0004,
            PheromoneType::Exploration => 0.0006,
            PheromoneType::Recruitment => 0.0002,
            PheromoneType::Territory => 0.00005,
            PheromoneType::Nest => 0.0003,
            PheromoneType::Water => 0.0005,
            PheromoneType::Enemy => 0.0001,
            PheromoneType::Quality => 0.0004,
            PheromoneType::Distance => 0.0007,
        };

        // Role-specific decay adjustments
        let role_multiplier = match role {
            "scout" => 1.2,  // Scout trails decay faster
            "worker" => 0.8, // Worker trails decay slower
            "soldier" => 1.0, // Soldier trails decay normally
            _ => 1.0,
        };

        base_decay * role_multiplier
    }

    fn get_expiration_for_type(&self, trail_type: &PheromoneType) -> i64 {
        match trail_type {
            PheromoneType::Food => 15000,
            PheromoneType::Danger => 30000,
            PheromoneType::Home => 12000,
            PheromoneType::Exploration => 8000,
            PheromoneType::Recruitment => 10000,
            PheromoneType::Territory => 50000,
            PheromoneType::Nest => 20000,
            PheromoneType::Water => 10000,
            PheromoneType::Enemy => 25000,
            PheromoneType::Quality => 15000,
            PheromoneType::Distance => 6000,
        }
    }

    fn get_pheromone_influence(
        &self,
        position: (f32, f32),
        colony_id: i32,
        radius: f32,
        ant_id: i32,
    ) -> (f32, f32) {
        // Get ant type for role-specific pheromone sensitivity
        let ant_type = self.cache.get_ant_type(&self.cache.ants.get(&ant_id).map(|ant| ant.ant_type_id).unwrap_or(0));
        let role = ant_type.as_ref().map(|at| at.role.as_str()).unwrap_or("worker");
        
        // Use the enhanced pheromone manager
        let pheromone_manager = PheromoneManager::new(self.cache.clone());
        pheromone_manager.get_pheromone_influence(position, colony_id, radius, role)
    }

    fn get_home_pheromone_influence(
        &self,
        position: (f32, f32),
        colony_id: i32,
        radius: f32,
        ant_id: i32,
    ) -> (f32, f32) {
        // Get ant type for role-specific pheromone sensitivity
        let ant_type = self.cache.get_ant_type(&self.cache.ants.get(&ant_id).map(|ant| ant.ant_type_id).unwrap_or(0));
        let role = ant_type.as_ref().map(|at| at.role.as_str()).unwrap_or("worker");
        
        let trails = self.cache.get_pheromone_trails_near_position(position, radius);
        let mut total_influence_x = 0.0;
        let mut total_influence_y = 0.0;
        let mut total_strength = 0.0;

        // Only consider home pheromone trails when returning to colony
        for trail in &trails {
            // Only consider trails from the same colony
            if trail.colony_id != colony_id {
                continue;
            }

            // Consider both Home and Nest pheromone trails
            if trail.trail_type == PheromoneType::Home || trail.trail_type == PheromoneType::Nest {
                let dx = trail.position.0 - position.0;
                let dy = trail.position.1 - position.1;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance > 0.0 && distance <= radius {
                    let normalized_distance = distance / radius;
                    let distance_decay = (-normalized_distance * 3.0).exp();
                    
                    // Give higher weight to own trails and role-specific sensitivity
                    let own_trail_multiplier = if trail.ant_id == ant_id { 2.0 } else { 1.0 };
                    let role_sensitivity = match role {
                        "worker" => 1.5,
                        "scout" => 1.2,
                        "soldier" => 1.0,
                        _ => 1.0,
                    };
                    
                    let influence = trail.strength * distance_decay * own_trail_multiplier * role_sensitivity;

                    let dir_x = dx / distance;
                    let dir_y = dy / distance;

                    total_influence_x += dir_x * influence;
                    total_influence_y += dir_y * influence;
                    total_strength += influence;
                }
            }
        }

        if total_strength == 0.0 {
            (0.0, 0.0)
        } else {
            (
                total_influence_y.atan2(total_influence_x),
                total_strength,
            )
        }
    }

    fn follow_recruitment_pheromone(&self, ant: &OptimizedAnt, direction: f32, strength: f32) -> anyhow::Result<()> {
        // Follow recruitment pheromone to help other ants
        let new_x = ant.position.0 + direction.cos() * ant.speed * strength;
        let new_y = ant.position.1 + direction.sin() * ant.speed * strength;
        
        self.cache.update_ant(ant.id, |ant| {
            ant.position = (new_x, new_y);
            ant.angle = direction;
            ant.state = AntState::Following;
        });
        
        Ok(())
    }

    fn follow_quality_pheromone(&self, ant: &OptimizedAnt, direction: f32, strength: f32) -> anyhow::Result<()> {
        // Follow quality pheromone to high-quality food sources
        let new_x = ant.position.0 + direction.cos() * ant.speed * strength;
        let new_y = ant.position.1 + direction.sin() * ant.speed * strength;
        
        self.cache.update_ant(ant.id, |ant| {
            ant.position = (new_x, new_y);
            ant.angle = direction;
            ant.state = AntState::SeekingFood;
        });
        
        Ok(())
    }

    fn avoid_territory(&self, ant: &OptimizedAnt, direction: f32, strength: f32) -> anyhow::Result<()> {
        // Move away from territory pheromone (enemy colony)
        let avoid_direction = direction + std::f32::consts::PI; // Opposite direction
        let new_x = ant.position.0 + avoid_direction.cos() * ant.speed * strength;
        let new_y = ant.position.1 + avoid_direction.sin() * ant.speed * strength;
        
        self.cache.update_ant(ant.id, |ant| {
            ant.position = (new_x, new_y);
            ant.angle = avoid_direction;
            ant.state = AntState::Wandering;
        });
        
        Ok(())
    }

    fn get_recruitment_pheromone_influence(
        &self,
        position: (f32, f32),
        colony_id: i32,
        radius: f32,
        _ant_id: i32,
    ) -> (f32, f32) {
        let trails = self.cache.get_pheromone_trails_near_position(position, radius);
        let mut total_influence_x = 0.0;
        let mut total_influence_y = 0.0;
        let mut total_strength = 0.0;

        for trail in &trails {
            if trail.colony_id != colony_id || trail.trail_type != PheromoneType::Recruitment {
                continue;
            }

            let dx = trail.position.0 - position.0;
            let dy = trail.position.1 - position.1;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 0.0 && distance <= radius {
                let normalized_distance = distance / radius;
                let distance_decay = (-normalized_distance * 2.5).exp(); // Recruitment has longer range
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
                total_influence_y.atan2(total_influence_x),
                total_strength,
            )
        }
    }

    fn get_quality_pheromone_influence(
        &self,
        position: (f32, f32),
        colony_id: i32,
        radius: f32,
        _ant_id: i32,
    ) -> (f32, f32) {
        let trails = self.cache.get_pheromone_trails_near_position(position, radius);
        let mut total_influence_x = 0.0;
        let mut total_influence_y = 0.0;
        let mut total_strength = 0.0;

        for trail in &trails {
            if trail.colony_id != colony_id || trail.trail_type != PheromoneType::Quality {
                continue;
            }

            let dx = trail.position.0 - position.0;
            let dy = trail.position.1 - position.1;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 0.0 && distance <= radius {
                let normalized_distance = distance / radius;
                let distance_decay = (-normalized_distance * 3.0).exp();
                
                // Quality trails are influenced by their quality rating
                let quality_factor = 1.0 + trail.quality_rating * 0.5;
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
                total_influence_y.atan2(total_influence_x),
                total_strength,
            )
        }
    }

    fn get_territory_pheromone_influence(
        &self,
        position: (f32, f32),
        colony_id: i32,
        radius: f32,
        _ant_id: i32,
    ) -> (f32, f32) {
        let trails = self.cache.get_pheromone_trails_near_position(position, radius);
        let mut total_influence_x = 0.0;
        let mut total_influence_y = 0.0;
        let mut total_strength = 0.0;

        for trail in &trails {
            // Only consider territory markers from OTHER colonies
            if trail.colony_id == colony_id || trail.trail_type != PheromoneType::Territory {
                continue;
            }

            let dx = trail.position.0 - position.0;
            let dy = trail.position.1 - position.1;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 0.0 && distance <= radius {
                let normalized_distance = distance / radius;
                let distance_decay = (-normalized_distance * 2.0).exp(); // Territory has medium range
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
                total_influence_y.atan2(total_influence_x),
                total_strength,
            )
        }
    }

    // Utility functions
    fn distance(&self, pos1: (f32, f32), pos2: (f32, f32)) -> f32 {
        let dx = pos1.0 - pos2.0;
        let dy = pos1.1 - pos2.1;
        (dx * dx + dy * dy).sqrt()
    }

    fn move_with_bounds(&self, position: (f32, f32), direction: f32, distance: f32) -> ((f32, f32), f32) {
        let mut new_x = position.0 + direction.cos() * distance;
        let mut new_y = position.1 + direction.sin() * distance;
        let mut new_direction = direction;
        let mut boundary_hit = false;

        // Handle horizontal boundaries with reflection
        if new_x < 0.0 {
            new_x = -new_x;
            new_direction = std::f32::consts::PI - new_direction;
            boundary_hit = true;
            tracing::debug!("🌍 Boundary collision: Ant hit left boundary at x={}, reflected to x={}", position.0, new_x);
        } else if new_x > self.cache.world_bounds.width {
            new_x = self.cache.world_bounds.width - (new_x - self.cache.world_bounds.width);
            new_direction = std::f32::consts::PI - new_direction;
            boundary_hit = true;
            tracing::debug!("🌍 Boundary collision: Ant hit right boundary at x={}, reflected to x={}", position.0, new_x);
        }

        // Handle vertical boundaries with reflection
        if new_y < 0.0 {
            new_y = -new_y;
            new_direction = -new_direction;
            boundary_hit = true;
            tracing::debug!("🌍 Boundary collision: Ant hit top boundary at y={}, reflected to y={}", position.1, new_y);
        } else if new_y > self.cache.world_bounds.height {
            new_y = self.cache.world_bounds.height - (new_y - self.cache.world_bounds.height);
            new_direction = -new_direction;
            boundary_hit = true;
            tracing::debug!("🌍 Boundary collision: Ant hit bottom boundary at y={}, reflected to y={}", position.1, new_y);
        }

        // Normalize direction
        new_direction = new_direction % (2.0 * std::f32::consts::PI);
        if new_direction < 0.0 {
            new_direction += 2.0 * std::f32::consts::PI;
        }

        if boundary_hit {
            tracing::debug!("🌍 Boundary reflection: Direction changed from {:.2} to {:.2}", direction, new_direction);
        }

        ((new_x, new_y), new_direction)
    }
}

#[derive(Debug, Clone)]
enum AntAction {
    Wander,
    Explore,
    Patrol,
    MoveToFood(i32),
    FollowPheromone,
    MoveToTarget,
    CollectFood(i32),
    ReturnToColony,
    DepositFood,
    None,
} 