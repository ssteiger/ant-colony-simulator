use crate::cache::SimulationCache;
use crate::models::*;
use rand::prelude::*;
use rand::rngs::SmallRng;
use rayon::prelude::*;
use std::sync::Arc;

pub struct AntBehaviorManager {
    cache: Arc<SimulationCache>,
}

impl AntBehaviorManager {
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

    fn process_ant_behavior(&self, ant: &FastAnt, current_tick: i64) -> anyhow::Result<()> {
        // Age the ant
        let new_age = ant.age_ticks + 1;
        
        // Check lifespan - get ant type for lifespan info
        if let Some(ant_type) = self.cache.get_ant_type(&ant.ant_type_id) {
            if new_age > ant_type.lifespan_ticks {
                self.kill_ant(ant.id, current_tick);
                return Ok(());
            }
        }

        // Energy decay
        let new_energy = (ant.energy - 1).max(0);
        if new_energy <= 0 {
            self.kill_ant(ant.id, current_tick);
            return Ok(());
        }

        // Determine next action based on current state and role
        let action = self.determine_ant_action(ant)?;
        
        // Execute the action
        self.execute_ant_action(ant, action, current_tick)?;

        Ok(())
    }

    fn determine_ant_action(&self, ant: &FastAnt) -> anyhow::Result<AntAction> {
        // Get ant type to determine role-based behavior
        let ant_type = self.cache.get_ant_type(&ant.ant_type_id)
            .ok_or_else(|| anyhow::anyhow!("Ant type {} not found", ant.ant_type_id))?;

        match ant.state {
            AntState::Wandering => {
                // Look for nearby food first
                let nearby_food = self.cache.get_food_sources_near_position(ant.position, 50.0)
                    .into_iter()
                    .find(|food| food.amount > 0);

                if let Some(food) = nearby_food {
                    // Role-based food prioritization
                    match ant_type.role.as_str() {
                        "soldier" => {
                            // Soldiers only collect very nearby food
                            let distance = self.distance(ant.position, food.position);
                            if distance < 20.0 {
                                return Ok(AntAction::SeekFood(food.id));
                            }
                        }
                        _ => {
                            return Ok(AntAction::SeekFood(food.id));
                        }
                    }
                }

                // Check for pheromone trails
                let search_radius = match ant_type.role.as_str() {
                    "scout" => 40.0,
                    "worker" => 30.0,
                    _ => 25.0,
                };

                let (direction, strength) = self.get_pheromone_influence(
                    ant.position,
                    ant.colony_id,
                    search_radius,
                );

                let follow_threshold = match ant_type.role.as_str() {
                    "scout" => 0.05,
                    "worker" => 0.1,
                    _ => 0.15,
                };

                if strength > follow_threshold {
                    return Ok(AntAction::FollowPheromone(direction, strength));
                }

                // Role-specific default behavior
                match ant_type.role.as_str() {
                    "scout" => Ok(AntAction::Explore),
                    "soldier" => Ok(AntAction::Patrol),
                    _ => Ok(AntAction::Wander),
                }
            }

            AntState::SeekingFood => {
                if let Some(target) = &ant.target {
                    if let Target::Food(food_id) = target {
                        // Check if we reached the food
                        if let Some(food) = self.cache.food_sources.get(food_id).map(|entry| entry.clone()) {
                            let distance = self.distance(ant.position, food.position);
                            if distance < 5.0 {
                                return Ok(AntAction::CollectFood(*food_id));
                            } else {
                                return Ok(AntAction::MoveToTarget);
                            }
                        }
                    }
                }
                // Lost target, wander
                Ok(AntAction::Wander)
            }

            AntState::CarryingFood => {
                Ok(AntAction::ReturnToColony)
            }

            AntState::ReturningToColony => {
                Ok(AntAction::ReturnToColony)
            }

            _ => Ok(AntAction::Wander),
        }
    }

    fn execute_ant_action(&self, ant: &FastAnt, action: AntAction, current_tick: i64) -> anyhow::Result<()> {
        match action {
            AntAction::Wander => self.move_ant_randomly(ant),
            AntAction::Explore => self.scout_explore(ant),
            AntAction::Patrol => self.soldier_patrol(ant),
            AntAction::SeekFood(food_id) => self.move_ant_towards_food(ant, food_id),
            AntAction::FollowPheromone(direction, strength) => self.follow_pheromone_trail(ant, direction, strength),
            AntAction::MoveToTarget => self.move_ant_towards_target(ant),
            AntAction::CollectFood(food_id) => self.collect_food(ant, food_id, current_tick),
            AntAction::ReturnToColony => self.move_ant_towards_colony(ant, current_tick),
        }
    }

    fn move_ant_randomly(&self, ant: &FastAnt) -> anyhow::Result<()> {
        let mut rng = SmallRng::from_entropy();
        
        // 5% chance to change direction
        let mut new_angle = ant.angle;
        if rng.gen_bool(0.05) {
            let angle_change = rng.gen_range(-0.15..0.15) * std::f32::consts::PI;
            new_angle += angle_change;
        }

        // Move forward
        let move_distance = ant.speed;
        let mut new_position = (
            ant.position.0 + new_angle.cos() * move_distance,
            ant.position.1 + new_angle.sin() * move_distance,
        );

        // Handle boundaries with reflection
        let bounds = &self.cache.world_bounds;
        if new_position.0 < 0.0 {
            new_position.0 = -new_position.0;
            new_angle = std::f32::consts::PI - new_angle;
        } else if new_position.0 > bounds.width {
            new_position.0 = bounds.width - (new_position.0 - bounds.width);
            new_angle = std::f32::consts::PI - new_angle;
        }

        if new_position.1 < 0.0 {
            new_position.1 = -new_position.1;
            new_angle = -new_angle;
        } else if new_position.1 > bounds.height {
            new_position.1 = bounds.height - (new_position.1 - bounds.height);
            new_angle = -new_angle;
        }

        // Normalize angle
        new_angle = new_angle % (2.0 * std::f32::consts::PI);
        if new_angle < 0.0 {
            new_angle += 2.0 * std::f32::consts::PI;
        }

        // Update ant
        self.cache.update_ant(ant.id, |ant| {
            ant.position = new_position;
            ant.angle = new_angle;
            ant.last_action_tick = self.cache.current_tick.try_read().map(|t| *t).unwrap_or(0);
        });

        Ok(())
    }

    fn scout_explore(&self, ant: &FastAnt) -> anyhow::Result<()> {
        let mut rng = SmallRng::from_entropy();
        
        // Scouts move in wider exploration patterns
        let mut new_angle = ant.angle;
        
        // 10% chance for major direction change (exploration)
        if rng.gen_bool(0.1) {
            new_angle = rng.gen_range(0.0..std::f32::consts::TAU);
        } else if rng.gen_bool(0.2) {
            // 20% chance for minor direction change
            let angle_change = rng.gen_range(-0.5..0.5);
            new_angle += angle_change;
        }

        // Scouts move faster than regular ants
        let move_distance = ant.speed * 1.2;
        let new_position = self.move_with_bounds(ant.position, new_angle, move_distance);

        // Lay weak exploration pheromone trail
        self.create_pheromone_trail(ant.position, ant.colony_id, PheromoneType::Exploration, 0.1, None);

        self.cache.update_ant(ant.id, |a| {
            a.position = new_position;
            a.angle = new_angle;
        });

        Ok(())
    }

    fn soldier_patrol(&self, ant: &FastAnt) -> anyhow::Result<()> {
        let colony = self.cache.get_colony(&ant.colony_id)
            .ok_or_else(|| anyhow::anyhow!("Colony not found"))?;

        let distance_from_colony = self.distance(ant.position, colony.center);
        let patrol_radius = 60.0;
        let mut rng = SmallRng::from_entropy();

        let direction = if distance_from_colony > patrol_radius {
            // Return to patrol area
            (colony.center.1 - ant.position.1).atan2(colony.center.0 - ant.position.0)
        } else {
            // Circular patrol
            let angle_to_colony = (ant.position.1 - colony.center.1).atan2(ant.position.0 - colony.center.0);
            angle_to_colony + std::f32::consts::PI / 3.0 + rng.gen_range(-0.25..0.25)
        };

        let new_position = self.move_with_bounds(ant.position, direction, ant.speed);

        self.cache.update_ant(ant.id, |a| {
            a.position = new_position;
            a.angle = direction;
        });

        Ok(())
    }

    fn move_ant_towards_food(&self, ant: &FastAnt, food_id: i32) -> anyhow::Result<()> {
        if let Some(food) = self.cache.food_sources.get(&food_id).map(|entry| entry.clone()) {
            let direction = (
                food.position.0 - ant.position.0,
                food.position.1 - ant.position.1,
            );
            let distance = (direction.0 * direction.0 + direction.1 * direction.1).sqrt();
            
            if distance > 0.0 {
                let normalized_direction = (direction.0 / distance, direction.1 / distance);
                let new_position = self.move_with_bounds(
                    ant.position,
                    normalized_direction.1.atan2(normalized_direction.0),
                    ant.speed,
                );

                self.cache.update_ant(ant.id, |a| {
                    a.position = new_position;
                    a.angle = normalized_direction.1.atan2(normalized_direction.0);
                    a.state = AntState::SeekingFood;
                    a.target = Some(Target::Food(food_id));
                });
            }
        } else {
            // Food source no longer exists, wander
            self.move_ant_randomly(ant)?;
        }
        Ok(())
    }

    fn follow_pheromone_trail(&self, ant: &FastAnt, direction: f32, strength: f32) -> anyhow::Result<()> {
        // Combine pheromone direction with some randomness
        let pheromone_weight = (strength * 2.0).min(0.8);
        let random_weight = 1.0 - pheromone_weight;
        let mut rng = SmallRng::from_entropy();
        let random_angle = rng.gen_range(-0.25..0.25) * std::f32::consts::PI;
        let combined_direction = direction + (random_angle * random_weight);

        // Speed boost on strong trails
        let speed_multiplier = 1.0 + (strength * 0.5);
        let move_distance = ant.speed * speed_multiplier;

        let new_position = self.move_with_bounds(ant.position, combined_direction, move_distance);

        // Check for food while following trail
        let nearby_food = self.cache.get_food_sources_near_position(new_position, 15.0)
            .into_iter()
            .find(|food| food.amount > 0);

        if let Some(food) = nearby_food {
            self.cache.update_ant(ant.id, |a| {
                a.position = new_position;
                a.angle = combined_direction;
                a.state = AntState::SeekingFood;
                a.target = Some(Target::Food(food.id));
            });
        } else {
            self.cache.update_ant(ant.id, |a| {
                a.position = new_position;
                a.angle = combined_direction;
            });
        }

        Ok(())
    }

    fn move_ant_towards_target(&self, ant: &FastAnt) -> anyhow::Result<()> {
        if let Some(target) = &ant.target {
            let target_position = match target {
                Target::Food(food_id) => {
                    self.cache.food_sources.get(food_id).map(|entry| entry.position)
                }
                Target::Colony(colony_id) => {
                    self.cache.get_colony(colony_id).map(|c| c.center)
                }
                Target::Position(x, y) => Some((*x, *y)),
            };

            if let Some(target_pos) = target_position {
                let direction = (target_pos.1 - ant.position.1).atan2(target_pos.0 - ant.position.0);
                let new_position = self.move_with_bounds(ant.position, direction, ant.speed);

                self.cache.update_ant(ant.id, |a| {
                    a.position = new_position;
                    a.angle = direction;
                });
            } else {
                // Target no longer exists
                self.cache.update_ant(ant.id, |a| {
                    a.state = AntState::Wandering;
                    a.target = None;
                });
            }
        } else {
            // No target, wander
            self.move_ant_randomly(ant)?;
        }

        Ok(())
    }

    fn collect_food(&self, ant: &FastAnt, food_id: i32, _current_tick: i64) -> anyhow::Result<()> {
        // Try to collect food from the source
        let mut food_collected = 0;
        
        self.cache.update_food_source(food_id, |food| {
            if food.amount > 0 {
                food_collected = 1.min(food.amount);
                food.amount -= food_collected;
            }
        });

        if food_collected > 0 {
            // Update ant to carry food
            self.cache.update_ant(ant.id, |a| {
                a.state = AntState::CarryingFood;
                a.target = Some(Target::Colony(a.colony_id));
                a.carried_resources.insert("food".to_string(), food_collected);
            });

            // Create food pheromone trail
            self.create_pheromone_trail(
                ant.position,
                ant.colony_id,
                PheromoneType::Food,
                0.8,
                Some(food_id),
            );

            tracing::debug!("🐜 Ant {} collected {} food from source {}", ant.id, food_collected, food_id);
        } else {
            // No food available, wander
            self.cache.update_ant(ant.id, |a| {
                a.state = AntState::Wandering;
                a.target = None;
            });
        }

        Ok(())
    }

    fn move_ant_towards_colony(&self, ant: &FastAnt, current_tick: i64) -> anyhow::Result<()> {
        if let Some(colony) = self.cache.get_colony(&ant.colony_id) {
            let distance_to_colony = self.distance(ant.position, colony.center);
            
            if distance_to_colony < colony.radius {
                // Reached colony, deposit food
                self.deposit_food(ant, current_tick)?;
            } else {
                // Move towards colony
                let direction = (
                    colony.center.0 - ant.position.0,
                    colony.center.1 - ant.position.1,
                );
                let distance = (direction.0 * direction.0 + direction.1 * direction.1).sqrt();
                
                if distance > 0.0 {
                    let normalized_direction = (direction.0 / distance, direction.1 / distance);
                    let new_position = self.move_with_bounds(
                        ant.position,
                        normalized_direction.1.atan2(normalized_direction.0),
                        ant.speed,
                    );

                    self.cache.update_ant(ant.id, |a| {
                        a.position = new_position;
                        a.angle = normalized_direction.1.atan2(normalized_direction.0);
                        a.state = AntState::ReturningToColony;
                    });

                    // Create home pheromone trail when carrying food
                    if ant.state == AntState::CarryingFood {
                        self.create_pheromone_trail(
                            ant.position,
                            ant.colony_id,
                            PheromoneType::Home,
                            0.5,
                            None,
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn deposit_food(&self, ant: &FastAnt, _current_tick: i64) -> anyhow::Result<()> {
        let food_amount: i32 = ant.carried_resources.values().sum();
        
        if food_amount > 0 {
            // Add food to colony resources
            self.cache.update_colony(ant.colony_id, |colony| {
                let current_food = colony.resources.get("food").unwrap_or(&0);
                colony.resources.insert("food".to_string(), current_food + food_amount);
            });

            // Clear ant's carried resources and set state to wandering
            self.cache.update_ant(ant.id, |a| {
                a.carried_resources.clear();
                a.state = AntState::Wandering;
                a.target = None;
            });

            tracing::debug!("🐜 Ant {} deposited {} food to colony {}", ant.id, food_amount, ant.colony_id);
        } else {
            // No food to deposit, just set to wandering
            self.cache.update_ant(ant.id, |a| {
                a.state = AntState::Wandering;
                a.target = None;
            });
        }

        Ok(())
    }

    fn kill_ant(&self, ant_id: i32, _current_tick: i64) {
        self.cache.update_ant(ant_id, |ant| {
            ant.state = AntState::Dead;
            ant.health = 0;
        });
        tracing::debug!("💀 Ant {} has died", ant_id);
    }

    fn create_pheromone_trail(&self, position: (f32, f32), colony_id: i32, trail_type: PheromoneType, strength: f32, target_food_id: Option<i32>) {
        let mut rng = SmallRng::from_entropy();
        let trail = FastPheromoneTrail {
            id: rng.gen::<i32>().abs(), // Temporary ID
            colony_id,
            trail_type,
            position,
            strength,
            decay_rate: 0.01,
            expires_at: 1000, // 1000 ticks from now
            target_food_id,
        };

        self.cache.insert_pheromone_trail(trail);
    }

    fn get_pheromone_influence(
        &self,
        position: (f32, f32),
        colony_id: i32,
        radius: f32,
    ) -> (f32, f32) {
        let trails = self.cache.get_pheromone_trails_near_position(position, radius);
        let mut total_influence_x = 0.0;
        let mut total_influence_y = 0.0;
        let mut total_strength = 0.0;

        for trail in trails {
            if trail.colony_id != colony_id || trail.strength <= 0.1 {
                continue;
            }

            let dx = trail.position.0 - position.0;
            let dy = trail.position.1 - position.1;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 0.0 && distance <= radius {
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

    // Utility functions
    fn distance(&self, pos1: (f32, f32), pos2: (f32, f32)) -> f32 {
        let dx = pos1.0 - pos2.0;
        let dy = pos1.1 - pos2.1;
        (dx * dx + dy * dy).sqrt()
    }

    fn move_with_bounds(&self, position: (f32, f32), direction: f32, distance: f32) -> (f32, f32) {
        let new_x = (position.0 + direction.cos() * distance).clamp(0.0, self.cache.world_bounds.width);
        let new_y = (position.1 + direction.sin() * distance).clamp(0.0, self.cache.world_bounds.height);
        (new_x, new_y)
    }
}

#[derive(Debug, Clone)]
enum AntAction {
    Wander,
    Explore,
    Patrol,
    SeekFood(i32),
    FollowPheromone(f32, f32), // direction, strength
    MoveToTarget,
    CollectFood(i32),
    ReturnToColony,
} 