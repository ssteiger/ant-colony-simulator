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

        tracing::debug!("🐜 Ant {} in state {:?} determining action", ant.id, ant.state);

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
                    ant.id,
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
                        //tracing::info!("🐜 Ant {} is a scout and is exploring", ant.id);
                        Ok(AntAction::Explore)
                    }
                    "soldier" => {
                        //tracing::info!("🐜 Ant {} is a soldier and is patrolling", ant.id);
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
                                // Stay focused on moving to the food source
                                return Ok(AntAction::MoveToTarget);
                            }
                        }
                    }
                }
                // Lost target, wander
                Ok(AntAction::Wander)
            }

            AntState::CarryingFood => {
                tracing::info!("🐜 Ant {} is carrying food and returning to colony", ant.id);
                Ok(AntAction::ReturnToColony)
            }

            _ => Ok(AntAction::Wander),
        }
    }

    fn execute_ant_action(&self, ant: &FastAnt, action: AntAction, current_tick: i64) -> anyhow::Result<()> {
        tracing::debug!("🐜 Ant {} executing action: {:?}", ant.id, action);
        
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
            
            tracing::debug!("🔄 Ant {} random direction change: {:.2} → {:.2} (change: {:.2}, reason: random wandering)", 
                ant.id, ant.angle, new_angle, angle_change);
        }

        // Move forward with new speed
        let move_distance = ant.speed;
        let ((new_x, new_y), new_angle) = self.move_with_bounds(ant.position, new_angle, move_distance);

        // Update ant
        self.cache.update_ant(ant.id, |ant| {
            ant.position = (new_x, new_y);
            ant.angle = new_angle;
            ant.speed = ant.speed;
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
            let old_angle = new_angle;
            new_angle = rng.gen_range(0.0..std::f32::consts::TAU);
            tracing::debug!("🧭 Ant {} scout major direction change: {:.2} → {:.2} (reason: exploration behavior)", 
                ant.id, old_angle, new_angle);
        } else if rng.gen_bool(0.2) {
            // 20% chance for minor direction change
            let old_angle = new_angle;
            let angle_change = rng.gen_range(-0.5..0.5);
            new_angle += angle_change;
            tracing::debug!("🧭 Ant {} scout minor direction change: {:.2} → {:.2} (change: {:.2}, reason: exploration adjustment)", 
                ant.id, old_angle, new_angle, angle_change);
        }

        // Scouts move faster than regular ants
        let move_distance = ant.speed * 1.2;
        let ((new_x, new_y), new_angle) = self.move_with_bounds(ant.position, new_angle, move_distance);

        // Lay weak exploration pheromone trail
        self.create_pheromone_trail((new_x, new_y), ant.colony_id, PheromoneType::Exploration, 0.1, None, ant.id);

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
            let return_direction = (colony.center.1 - ant.position.1).atan2(colony.center.0 - ant.position.0);
            tracing::debug!("🛡️ Ant {} soldier direction change: {:.2} → {:.2} (reason: returning to patrol area, distance from colony: {:.1})", 
                ant.id, ant.angle, return_direction, distance_from_colony);
            return_direction
        } else {
            // Circular patrol
            let angle_to_colony = (ant.position.1 - colony.center.1).atan2(ant.position.0 - colony.center.0);
            let patrol_direction = angle_to_colony + std::f32::consts::PI / 3.0 + rng.gen_range(-0.25..0.25);
            tracing::debug!("🛡️ Ant {} soldier direction change: {:.2} → {:.2} (reason: circular patrol around colony)", 
                ant.id, ant.angle, patrol_direction);
            patrol_direction
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
                let mut new_angle = normalized_direction.1.atan2(normalized_direction.0);

                // Blend in pheromone influence if available
                let (trail_dir, strength) =
                    self.get_pheromone_influence(ant.position, ant.colony_id, 30.0, ant.id);
                if strength > 0.05 {
                    let weight = strength.min(1.0);
                    let blended_x = new_angle.cos() * (1.0 - weight) + trail_dir.cos() * weight;
                    let blended_y = new_angle.sin() * (1.0 - weight) + trail_dir.sin() * weight;
                    let old_angle = new_angle;
                    new_angle = blended_y.atan2(blended_x);
                    tracing::debug!("🍎 Ant {} direction blended with pheromone: {:.2} → {:.2} (pheromone strength: {:.2}, weight: {:.2})", 
                        ant.id, old_angle, new_angle, strength, weight);
                } else {
                    tracing::debug!("🍎 Ant {} direction change: {:.2} → {:.2} (reason: moving towards food source {})", 
                        ant.id, ant.angle, new_angle, food_id);
                }

                let ((new_x, new_y), new_angle) =
                    self.move_with_bounds(ant.position, new_angle, ant.speed);

                self.cache.update_ant(ant.id, |a| {
                    a.position = (new_x, new_y);
                    a.angle = new_angle;
                    a.state = AntState::SeekingFood;
                    a.target = Some(Target::Food(food_id));
                });

                // Drop a home pheromone trail while moving towards food
                self.create_pheromone_trail(
                    (new_x, new_y),
                    ant.colony_id,
                    PheromoneType::Home,
                    0.3,
                    None,
                    ant.id,
                );
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

        tracing::debug!("🦨 Ant {} following pheromone trail: {:.2} → {:.2} (pheromone direction: {:.2}, strength: {:.2}, random factor: {:.2})", 
            ant.id, ant.angle, combined_direction, direction, strength, random_angle);

        // Speed boost on strong trails
        let speed_multiplier = 1.0 + (strength * 0.5);
        let move_distance = ant.speed * speed_multiplier;
        
        if speed_multiplier != 1.0 {
            tracing::debug!("⚡ Ant {} speed boost on pheromone trail: {:.2} → {:.2} (multiplier: {:.2}, reason: strong pheromone trail)", 
                ant.id, ant.speed, ant.speed * speed_multiplier, speed_multiplier);
        }

        let ((new_x, new_y), new_angle) = self.move_with_bounds(ant.position, combined_direction, move_distance);

        // Check for food while following trail
        let nearby_food = self.cache.get_food_sources_near_position((new_x, new_y), 15.0)
            .into_iter()
            .find(|food| food.amount > 0);

        if let Some(food) = nearby_food {
            // Only switch to food seeking if we're not already seeking food
            // or if we're seeking a different food source
            let should_switch = match &ant.target {
                Some(Target::Food(current_food_id)) => *current_food_id != food.id,
                _ => true,
            };

            if should_switch {
                self.cache.update_ant(ant.id, |a| {
                    a.position = (new_x, new_y);
                    a.angle = new_angle;
                    a.state = AntState::SeekingFood;
                    a.target = Some(Target::Food(food.id));
                });
            } else {
                // Continue with current food seeking behavior
                self.cache.update_ant(ant.id, |a| {
                    a.position = (new_x, new_y);
                    a.angle = new_angle;
                });
            }
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

            if angle_change != 0.0 {
                tracing::debug!("🎯 Ant {} turning towards target: {:.2} → {:.2} (change: {:.2}, desired: {:.2}, reason: target navigation)", 
                    ant.id, ant.angle, new_angle, angle_change, desired_angle);
            }

            // Adjust speed based on distance to target
            let distance = self.distance(ant.position, target_pos);
            let mut new_speed = ant.speed;
            
            // Get base speed from ant type for proper limits
            let base_speed = self.cache.get_ant_type(&ant.ant_type_id)
                .map(|ant_type| ant_type.base_speed as f32)
                .unwrap_or(1.0);
            
            if distance < 20.0 {
                // Slow down when approaching target
                let old_speed = new_speed;
                new_speed = (new_speed - 0.1).max(base_speed * 0.5);
                tracing::debug!("🐌 Ant {} slowing down near target: {:.2} → {:.2} (distance: {:.1}, reason: approaching target)", 
                    ant.id, old_speed, new_speed, distance);
            } else {
                // Accelerate when far from target
                let old_speed = new_speed;
                new_speed = (new_speed + 0.2).min(base_speed * 1.5);
                tracing::debug!("⚡ Ant {} accelerating towards target: {:.2} → {:.2} (distance: {:.1}, reason: far from target)", 
                    ant.id, old_speed, new_speed, distance);
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
                // Preserve the food source as the target so we can return later
                a.target = Some(Target::Food(food_id));
                a.carried_resources.insert("food".to_string(), food_collected);
                a.last_food_source_id = Some(food_id);
            });

            // Create food pheromone trail
            self.create_pheromone_trail(
                ant.position,
                ant.colony_id,
                PheromoneType::Food,
                0.8,
                Some(food_id),
                ant.id,
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
                    let mut new_angle = normalized_direction.1.atan2(normalized_direction.0);

                    // Only blend with home pheromone trails when returning to colony
                    let (trail_dir, strength) =
                        self.get_home_pheromone_influence(ant.position, ant.colony_id, 30.0, ant.id);
                    if strength > 0.05 {
                        let weight = strength.min(0.3); // Reduced weight to prioritize colony direction
                        let blended_x = new_angle.cos() * (1.0 - weight) + trail_dir.cos() * weight;
                        let blended_y = new_angle.sin() * (1.0 - weight) + trail_dir.sin() * weight;
                        let old_angle = new_angle;
                        new_angle = blended_y.atan2(blended_x);
                        tracing::debug!("🏠 Ant {} direction blended with home pheromone while returning: {:.2} → {:.2} (pheromone strength: {:.2}, weight: {:.2})", 
                            ant.id, old_angle, new_angle, strength, weight);
                    } else {
                        tracing::debug!("🏠 Ant {} direction change: {:.2} → {:.2} (reason: returning to colony)", 
                            ant.id, ant.angle, new_angle);
                    }

                    let ((new_x, new_y), new_angle) =
                        self.move_with_bounds(ant.position, new_angle, ant.speed);

                    tracing::info!("🐜 Ant {} moving from ({}, {}) to ({}, {}) towards colony", ant.id, ant.position.0, ant.position.1, new_x, new_y);

                    self.cache.update_ant(ant.id, |a| {
                        a.position = (new_x, new_y);
                        a.angle = new_angle;
                    });

                    // Create food pheromone trail while returning with food
                    if ant.state == AntState::CarryingFood {
                        self.create_pheromone_trail(
                            (new_x, new_y),
                            ant.colony_id,
                            PheromoneType::Food,
                            0.5,
                            None,
                            ant.id,
                        );
                    }
                } else {
                    tracing::warn!("🐜 Ant {} is at the same position as colony center, this shouldn't happen", ant.id);
                }
            }
        } else {
            tracing::warn!("🐜 Ant {} cannot find its colony {}", ant.id, ant.colony_id);
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

            // Determine the food source we last visited
            let food_source_id = match &ant.target {
                Some(Target::Food(id)) => Some(*id),
                _ => ant.last_food_source_id,
            };

            tracing::info!("food_source_id: {:?}", food_source_id);
            
            // Clear ant's carried resources
            self.cache.update_ant(ant.id, |a| {
                a.carried_resources.clear();
                
                // If we have a valid food source ID, return to it
                if let Some(food_id) = food_source_id {
                    let remaining = self
                        .cache
                        .food_sources
                        .get(&food_id)
                        .map(|f| f.amount)
                        .unwrap_or(0);
                    if remaining > 0 {
                        tracing::info!("🐜 Ant {} deposited {} food to colony {} and is returning to food source {}", ant.id, food_amount, ant.colony_id, food_id);
                        a.state = AntState::SeekingFood;
                        a.target = Some(Target::Food(food_id));
                        a.last_food_source_id = Some(food_id);
                    } else {
                        tracing::info!("🐜 Ant {} deposited food but source {} is depleted", ant.id, food_id);
                        a.state = AntState::Wandering;
                        a.target = None;
                        a.last_food_source_id = None;
                    }
                } else {
                    // No food source to return to, start wandering
                    tracing::info!("🐜 Ant {} deposited {} food to colony {} and is starting to wander", ant.id, food_amount, ant.colony_id);
                    a.state = AntState::Wandering;
                    a.target = None;
                    a.last_food_source_id = None;
                }
            });

            // Reinforce home pheromone at the colony entrance
            if let Some(colony) = self.cache.get_colony(&ant.colony_id) {
                self.create_pheromone_trail(
                    colony.center,
                    ant.colony_id,
                    PheromoneType::Home,
                    0.6,
                    None,
                    ant.id,
                );
            }
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

    fn create_pheromone_trail(&self, position: (f32, f32), colony_id: i32, trail_type: PheromoneType, strength: f32, target_food_id: Option<i32>, ant_id: i32) {
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
            ant_id,
        };

        self.cache.insert_pheromone_trail(trail);
    }

    fn get_pheromone_influence(
        &self,
        position: (f32, f32),
        colony_id: i32,
        radius: f32,
        ant_id: i32,
    ) -> (f32, f32) {
        let trails = self.cache.get_pheromone_trails_near_position(position, radius);
        let mut total_influence_x = 0.0;
        let mut total_influence_y = 0.0;
        let mut total_strength = 0.0;

        // First pass: Look for own food trails with higher weight
        for trail in &trails {
            // Only consider trails from the same colony
            if trail.colony_id != colony_id {
                continue;
            }

            // Prioritize own food trails
            if trail.trail_type == PheromoneType::Food {
                let dx = trail.position.0 - position.0;
                let dy = trail.position.1 - position.1;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance > 0.0 && distance <= radius {
                    let normalized_distance = distance / radius;
                    let distance_decay = (-normalized_distance * 3.0).exp();
                    
                    // Give higher weight to own trails
                    let own_trail_multiplier = if trail.ant_id == ant_id { 2.0 } else { 1.0 };
                    let influence = trail.strength * distance_decay * own_trail_multiplier;

                    let dir_x = dx / distance;
                    let dir_y = dy / distance;

                    total_influence_x += dir_x * influence;
                    total_influence_y += dir_y * influence;
                    total_strength += influence;
                }
            }
        }

        // If we found strong own trails, return immediately
        if total_strength > 0.2 {
            return (
                total_influence_y.atan2(total_influence_x),
                total_strength,
            );
        }

        // Second pass: Look for other food trails if no strong own trails found
        for trail in trails {
            // Only consider trails from the same colony
            if trail.colony_id != colony_id {
                continue;
            }

            // Consider other food pheromone trails
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

    fn get_home_pheromone_influence(
        &self,
        position: (f32, f32),
        colony_id: i32,
        radius: f32,
        ant_id: i32,
    ) -> (f32, f32) {
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

            // Only consider home pheromone trails
            if trail.trail_type == PheromoneType::Home {
                let dx = trail.position.0 - position.0;
                let dy = trail.position.1 - position.1;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance > 0.0 && distance <= radius {
                    let normalized_distance = distance / radius;
                    let distance_decay = (-normalized_distance * 3.0).exp();
                    
                    // Give higher weight to own trails
                    let own_trail_multiplier = if trail.ant_id == ant_id { 2.0 } else { 1.0 };
                    let influence = trail.strength * distance_decay * own_trail_multiplier;

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
    SeekFood(i32),
    FollowPheromone(f32, f32), // direction, strength
    MoveToTarget,
    CollectFood(i32),
    ReturnToColony,
} 