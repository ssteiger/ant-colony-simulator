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
    // Movement constraints as associated constants
    const MAX_TURN_RATE: f32 = 0.1; // Maximum radians per tick
    const ACCELERATION: f32 = 0.2; // Speed change per tick
    const DECELERATION: f32 = 0.1; // Speed change per tick

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
            tracing::info!("🐜 Ant {} has died", ant.id);
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
                    tracing::info!("🐜 Ant {} found food {}", ant.id, food.id);
                    // Role-based food prioritization
                    match ant_type.role.as_str() {
                        "soldier" => {
                            tracing::info!("🐜 Ant {} is a soldier", ant.id);
                            // Soldiers only collect very nearby food
                            let distance = self.distance(ant.position, food.position);
                            if distance < 20.0 {
                                return Ok(AntAction::SeekFood(food.id));
                            }
                        }
                        _ => {
                            tracing::info!("🐜 Ant {} is a {} and is seeking food", ant.id, ant_type.role);
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

                // First check for food pheromone trails
                let (food_direction, food_strength) = self.get_pheromone_influence(
                    ant.position,
                    ant.colony_id,
                    search_radius,
                );

                let follow_threshold = match ant_type.role.as_str() {
                    "scout" => 0.05,
                    "worker" => 0.1,
                    _ => 0.15,
                };

                if food_strength > follow_threshold {
                    tracing::info!("🐜 Ant {} is following food pheromone trail", ant.id);
                    return Ok(AntAction::FollowPheromone(food_direction, food_strength));
                }

                // Role-specific default behavior
                match ant_type.role.as_str() {
                    "scout" => {
                        tracing::info!("🐜 Ant {} is a scout and is exploring", ant.id);
                        Ok(AntAction::Explore)
                    }
                    "soldier" => {
                        tracing::info!("🐜 Ant {} is a soldier and is patrolling", ant.id);
                        Ok(AntAction::Patrol)
                    }
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
        
        // Calculate new angle with turn rate limit
        let mut new_angle = ant.angle;
        if rng.gen_bool(0.05) {
            let desired_angle_change = rng.gen_range(-0.15..0.15) * std::f32::consts::PI;
            // Limit the turn rate
            let angle_change = desired_angle_change.max(-Self::MAX_TURN_RATE).min(Self::MAX_TURN_RATE);
            new_angle += angle_change;
        }

        // Gradually adjust speed
        let mut new_speed = ant.speed;
        if rng.gen_bool(0.1) {
            // Randomly accelerate or decelerate
            if rng.gen_bool(0.5) {
                new_speed = (new_speed + Self::ACCELERATION).min(ant.speed * 1.5);
            } else {
                new_speed = (new_speed - Self::DECELERATION).max(ant.speed * 0.5);
            }
        }

        // Move forward with new speed
        let move_distance = new_speed;
        let ((new_x, new_y), new_angle) = self.move_with_bounds(ant.position, new_angle, move_distance);

        // Update ant
        self.cache.update_ant(ant.id, |ant| {
            ant.position = (new_x, new_y);
            ant.angle = new_angle;
            ant.speed = new_speed;
            ant.state = AntState::Wandering;
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
        let ((new_x, new_y), new_angle) = self.move_with_bounds(ant.position, new_angle, move_distance);

        // Lay weak exploration pheromone trail
        self.create_pheromone_trail((new_x, new_y), ant.colony_id, PheromoneType::Exploration, 0.1, None);

        self.cache.update_ant(ant.id, |a| {
            a.position = (new_x, new_y);
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

        let ((new_x, new_y), new_angle) = self.move_with_bounds(ant.position, direction, ant.speed);

        self.cache.update_ant(ant.id, |a| {
            a.position = (new_x, new_y);
            a.angle = new_angle;
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
                let new_angle = normalized_direction.1.atan2(normalized_direction.0);
                let ((new_x, new_y), new_angle) = self.move_with_bounds(
                    ant.position,
                    new_angle,
                    ant.speed,
                );

                self.cache.update_ant(ant.id, |a| {
                    a.position = (new_x, new_y);
                    a.angle = new_angle;
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

        let ((new_x, new_y), new_angle) = self.move_with_bounds(ant.position, combined_direction, move_distance);

        // Check for food while following trail
        let nearby_food = self.cache.get_food_sources_near_position((new_x, new_y), 15.0)
            .into_iter()
            .find(|food| food.amount > 0);

        if let Some(food) = nearby_food {
            self.cache.update_ant(ant.id, |a| {
                a.position = (new_x, new_y);
                a.angle = new_angle;
                a.state = AntState::SeekingFood;
                a.target = Some(Target::Food(food.id));
            });
        } else {
            self.cache.update_ant(ant.id, |a| {
                a.position = (new_x, new_y);
                a.angle = new_angle;
            });
        }

        Ok(())
    }

    fn move_ant_towards_target(&self, ant: &FastAnt) -> anyhow::Result<()> {
        if let Some(target) = &ant.target {
            let target_pos = match target {
                Target::Food(food_id) => {
                    if let Some(food) = self.cache.food_sources.get(food_id) {
                        food.position
                    } else {
                        return Ok(());
                    }
                }
                Target::Position(x, y) => (*x as f32, *y as f32),
                Target::Colony(colony_id) => {
                    if let Some(colony) = self.cache.colonies.get(colony_id) {
                        colony.center
                    } else {
                        return Ok(());
                    }
                }
            };

            // Calculate desired angle to target
            let dx = target_pos.0 - ant.position.0;
            let dy = target_pos.1 - ant.position.1;
            let desired_angle = dy.atan2(dx);

            // Calculate angle difference
            let mut angle_diff = desired_angle - ant.angle;
            while angle_diff > std::f32::consts::PI {
                angle_diff -= 2.0 * std::f32::consts::PI;
            }
            while angle_diff < -std::f32::consts::PI {
                angle_diff += 2.0 * std::f32::consts::PI;
            }

            // Limit turn rate
            let angle_change = angle_diff.max(-Self::MAX_TURN_RATE).min(Self::MAX_TURN_RATE);
            let new_angle = ant.angle + angle_change;

            // Adjust speed based on distance to target
            let distance = self.distance(ant.position, target_pos);
            let mut new_speed = ant.speed;
            
            if distance < 20.0 {
                // Slow down when approaching target
                new_speed = (new_speed - Self::DECELERATION).max(ant.speed * 0.5);
            } else {
                // Accelerate when far from target
                new_speed = (new_speed + Self::ACCELERATION).min(ant.speed * 1.5);
            }

            // Move forward
            let ((new_x, new_y), new_angle) = self.move_with_bounds(ant.position, new_angle, new_speed);

            // Update ant
            self.cache.update_ant(ant.id, |ant| {
                ant.position = (new_x, new_y);
                ant.angle = new_angle;
                ant.speed = new_speed;
                ant.last_action_tick = self.cache.current_tick.try_read().map(|t| *t).unwrap_or(0);
            });
        }

        Ok(())
    }

    fn collect_food(&self, ant: &FastAnt, food_id: i32, _current_tick: i64) -> anyhow::Result<()> {
        // Try to collect food from the source
        let mut food_collected = 0;
        
        self.cache.update_food_source(food_id, |food| {
            if food.amount > 0 {
                // Collect up to 5 food units per collection
                food_collected = 5.min(food.amount);
                food.amount -= food_collected;
                tracing::info!("🍎 Food source {} amount reduced to {}", food_id, food.amount);
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

            tracing::info!("🐜 Ant {} collected {} food from source {}", ant.id, food_collected, food_id);
        } else {
            // No food available, wander
            self.cache.update_ant(ant.id, |a| {
                a.state = AntState::Wandering;
                a.target = None;
            });
            tracing::info!("🍎 Food source {} is depleted", food_id);
        }

        Ok(())
    }

    fn move_ant_towards_colony(&self, ant: &FastAnt, current_tick: i64) -> anyhow::Result<()> {
        if let Some(colony) = self.cache.get_colony(&ant.colony_id) {
            let distance_to_colony = self.distance(ant.position, colony.center);
            tracing::info!("🐜 Ant {} is moving towards colony at distance {} from position ({}, {})", ant.id, distance_to_colony, ant.position.0, ant.position.1);
            
            if distance_to_colony < colony.radius {
                tracing::info!("🐜 Ant {} reached colony and is depositing food", ant.id);
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
                    let new_angle = normalized_direction.1.atan2(normalized_direction.0);
                    let ((new_x, new_y), new_angle) = self.move_with_bounds(
                        ant.position,
                        new_angle,
                        ant.speed,
                    );

                    self.cache.update_ant(ant.id, |a| {
                        a.position = (new_x, new_y);
                        a.angle = new_angle;
                    });

                    // Create home pheromone trail when carrying food
                    if ant.state == AntState::CarryingFood {
                        tracing::info!("🐜 Ant {} is carrying food and is creating home pheromone trail", ant.id);
                        self.create_pheromone_trail(
                            (new_x, new_y),
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
            decay_rate: 0.0005, // Reduced from 0.005 to 0.0005 (0.05% per tick)
            expires_at: 20000, // Increased from 2000 to 20000 ticks
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
            // Only consider trails from the same colony
            if trail.colony_id != colony_id {
                continue;
            }

            // For ants not carrying food, prioritize food pheromone trails
            if trail.trail_type == PheromoneType::Food && trail.strength > 0.1 {
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

        // Handle horizontal boundaries with reflection
        if new_x < 0.0 {
            new_x = -new_x;
            new_direction = std::f32::consts::PI - new_direction;
        } else if new_x > self.cache.world_bounds.width {
            new_x = self.cache.world_bounds.width - (new_x - self.cache.world_bounds.width);
            new_direction = std::f32::consts::PI - new_direction;
        }

        // Handle vertical boundaries with reflection
        if new_y < 0.0 {
            new_y = -new_y;
            new_direction = -new_direction;
        } else if new_y > self.cache.world_bounds.height {
            new_y = self.cache.world_bounds.height - (new_y - self.cache.world_bounds.height);
            new_direction = -new_direction;
        }

        // Normalize direction
        new_direction = new_direction % (2.0 * std::f32::consts::PI);
        if new_direction < 0.0 {
            new_direction += 2.0 * std::f32::consts::PI;
        }

        ((new_x, new_y), new_direction)
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