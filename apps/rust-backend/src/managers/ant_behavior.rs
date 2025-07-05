use crate::models::*;
use rand::prelude::*;
use bevy::prelude::*;
use tracing::{debug, info, warn, trace};

// ============================================================================
// BEVY SYSTEMS
// ============================================================================

pub fn ant_movement_system(
    mut ants: Query<(
        &mut AntPhysics,
        &AntHealth,
        &AntState,
        &AntTarget,
        &mut Transform,
        Entity,
    ), With<Ant>>,
    time: Res<Time>,
) {
    for (mut physics, health, state, target, mut transform, entity) in ants.iter_mut() {
        if health.health <= 0.0 {
            trace!("Ant {:?} is dead, skipping movement", entity);
            continue; // Skip dead ants
        }

        let delta_time = time.delta_seconds();
        let old_position = physics.position;
        
        // Calculate target velocity based on state and target
        let target_velocity = match (state, target) {
            (AntState::Wandering, _) => {
                // Random movement for wandering ants
                let mut rng = rand::thread_rng();
                let angle = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
                let velocity = Vec2::new(angle.cos(), angle.sin()) * physics.max_speed * 0.3;
                trace!("Ant {:?} wandering: angle={:.2}, velocity=({:.2}, {:.2})", 
                       entity, angle, velocity.x, velocity.y);
                velocity
            }
            (AntState::SeekingFood, AntTarget::Food(_)) => {
                // Move towards food target
                trace!("Ant {:?} seeking food target", entity);
                Vec2::ZERO // Will be calculated based on target position
            }
            (AntState::CarryingFood, AntTarget::Colony(_)) => {
                // Move towards colony
                trace!("Ant {:?} carrying food to colony", entity);
                Vec2::ZERO // Will be calculated based on target position
            }
            (AntState::Following, AntTarget::Position(pos)) => {
                // Move towards specific position
                let direction = (*pos - physics.position).normalize();
                let velocity = direction * physics.max_speed;
                trace!("Ant {:?} following to position ({:.2}, {:.2}), velocity=({:.2}, {:.2})", 
                       entity, pos.x, pos.y, velocity.x, velocity.y);
                velocity
            }
            _ => {
                trace!("Ant {:?} in state {:?} with target {:?}, no movement", entity, state, target);
                Vec2::ZERO
            },
        };

        // Apply acceleration towards target velocity
        let velocity_diff = target_velocity - physics.velocity;
        let acceleration = velocity_diff.normalize() * physics.acceleration * delta_time;
        
        physics.velocity += acceleration;
        
        // Clamp velocity to max speed
        if physics.velocity.length() > physics.max_speed {
            physics.velocity = physics.velocity.normalize() * physics.max_speed;
        }

        // Update position
        let velocity = physics.velocity;
        physics.position += velocity * delta_time;
        
        // Update transform
        transform.translation.x = physics.position.x;
        transform.translation.y = physics.position.y;
        
        // Update rotation based on velocity direction
        if physics.velocity.length() > 0.1 {
            physics.rotation = physics.velocity.y.atan2(physics.velocity.x);
            transform.rotation = Quat::from_rotation_z(physics.rotation);
        }

        // Log significant movement
        let distance_moved = old_position.distance(physics.position);
        if distance_moved > 1.0 {
            debug!("Ant {:?} moved {:.2} units from ({:.2}, {:.2}) to ({:.2}, {:.2})", 
                   entity, distance_moved, old_position.x, old_position.y, 
                   physics.position.x, physics.position.y);
        }
    }
}

pub fn ant_health_system(
    mut ants: Query<(&mut AntHealth, &AntPhysics, Entity), With<Ant>>,
    simulation_state: Res<SimulationState>,
) {
    for (mut health, _physics, entity) in ants.iter_mut() {
        // Age the ant
        health.age_ticks += 1;
        
        // Energy decay
        let old_energy = health.energy;
        //health.energy = (health.energy - 1.0).max(0.0);
        
        // Health decay if no energy
        let old_health = health.health;
        if health.energy <= 0.0 {
            //health.health = (health.health - 5.0).max(0.0);
        }

        // Log significant health changes
        if health.energy != old_energy && health.energy % 10.0 < 1.0 {
            debug!("Ant {:?} energy: {:.1} -> {:.1}", entity, old_energy, health.energy);
        }
        
        if health.health != old_health {
            if health.health <= 0.0 {
                info!("Ant {:?} has died! Health: {:.1} -> {:.1}", entity, old_health, health.health);
            } else if health.health < old_health {
                warn!("Ant {:?} health decreased: {:.1} -> {:.1}", entity, old_health, health.health);
            } else {
                debug!("Ant {:?} health increased: {:.1} -> {:.1}", entity, old_health, health.health);
            }
        }

        // Log age milestones
        if health.age_ticks % 100 == 0 {
            debug!("Ant {:?} age: {} ticks, health: {:.1}, energy: {:.1}", 
                   entity, health.age_ticks, health.health, health.energy);
        }
    }
}

/// System to despawn dead ants
pub fn despawn_dead_ants_system(
    mut commands: Commands,
    dead_ants: Query<(Entity, &AntHealth), (With<Ant>, Changed<AntHealth>)>,
) {
    for (entity, health) in dead_ants.iter() {
        if health.health <= 0.0 {
            info!("Despawning dead ant {:?}", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Comprehensive cleanup system for dead ants (runs every frame)
pub fn cleanup_dead_ants_system(
    mut commands: Commands,
    all_ants: Query<(Entity, &AntHealth), With<Ant>>,
) {
    for (entity, health) in all_ants.iter() {
        if health.health <= 0.0 {
            debug!("Cleaning up dead ant {:?}", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn ant_ai_system(
    mut ants: Query<(
        &mut AntState,
        &mut AntTarget,
        &AntHealth,
        &CarriedResources,
        &AntMemory,
        &AntPhysics,
        Entity,
    ), With<Ant>>,
    food_sources: Query<&FoodSourceProperties, With<FoodSource>>,
    colonies: Query<&ColonyProperties, With<Colony>>,
    pheromones: Query<&PheromoneProperties, With<PheromoneTrail>>,
) {
    for (mut state, mut target, health, resources, memory, physics, entity) in ants.iter_mut() {
        let old_state = state.clone();
        
        // Simple AI logic - can be enhanced with Big Brain
        if health.energy < 20.0 && !resources.resources.is_empty() {
            *state = AntState::CarryingFood;
            if old_state != *state {
                info!("Ant {:?} switching to CarryingFood (energy: {:.1}, resources: {:.1})", 
                      entity, health.energy, resources.current_weight);
            }
        } else if health.energy > 50.0 && resources.resources.is_empty() {
            *state = AntState::SeekingFood;
            if old_state != *state {
                info!("Ant {:?} switching to SeekingFood (energy: {:.1})", entity, health.energy);
            }
        } else {
            *state = AntState::Wandering;
            if old_state != *state {
                debug!("Ant {:?} switching to Wandering (energy: {:.1}, resources: {:.1})", 
                       entity, health.energy, resources.current_weight);
            }
        }

        // Log AI decision making
        trace!("Ant {:?} AI decision: state={:?}, energy={:.1}, resources={:.1}, known_food_sources={}", 
               entity, *state, health.energy, resources.current_weight, memory.known_food_sources.len());
    }
}

pub fn ant_food_interaction_system(
    mut ants: Query<(
        &mut CarriedResources,
        &mut AntHealth,
        &AntPhysics,
        &mut AntTarget,
        &mut AntMemory,
        Entity,
    ), With<Ant>>,
    mut food_sources: Query<(&mut FoodSourceProperties, &Transform, Entity), With<FoodSource>>,
) {
    for (mut resources, mut health, physics, mut target, mut memory, ant_entity) in ants.iter_mut() {
        for (mut food, food_transform, food_entity) in food_sources.iter_mut() {
            let distance = physics.position.distance(food_transform.translation.truncate());
            
            if distance < 10.0 && food.amount > 0.0 {
                // Ant is close enough to collect food
                let collect_amount = (food.amount * 0.1).min(resources.capacity - resources.current_weight);
                
                if collect_amount > 0.0 {
                    let old_food_amount = food.amount;
                    let old_resources_weight = resources.current_weight;
                    
                    // Collect food
                    food.amount -= collect_amount;
                    *resources.resources.entry(food.food_type.clone()).or_insert(0.0) += collect_amount;
                    resources.current_weight += collect_amount;
                    
                    info!("Ant {:?} collected {:.2} {} from food source {:?} (distance: {:.1})", 
                          ant_entity, collect_amount, food.food_type, food_entity, distance);
                    debug!("Food source {:?}: {:.2} -> {:.2}, Ant {:?} resources: {:.2} -> {:.2}", 
                           food_entity, old_food_amount, food.amount, ant_entity, old_resources_weight, resources.current_weight);
                    
                    // Update ant state
                    *target = AntTarget::None;
                    
                    // Remember this food source
                    if !memory.known_food_sources.contains(&food_entity) {
                        memory.known_food_sources.push(food_entity);
                        debug!("Ant {:?} learned about new food source {:?}", ant_entity, food_entity);
                    }
                    memory.last_food_source = Some(food_entity);
                } else {
                    trace!("Ant {:?} near food source {:?} but can't collect (full capacity or no food)", 
                           ant_entity, food_entity);
                }
            } else if distance < 15.0 {
                trace!("Ant {:?} approaching food source {:?} (distance: {:.1}, food: {:.2})", 
                       ant_entity, food_entity, distance, food.amount);
            }
        }
    }
}

pub fn ant_colony_interaction_system(
    mut ants: Query<(
        &mut CarriedResources,
        &mut AntHealth,
        &AntPhysics,
        &mut AntTarget,
        Entity,
    ), With<Ant>>,
    mut colonies: Query<(&mut ColonyResources, &ColonyProperties, &Transform, Entity), With<Colony>>,
) {
    for (mut resources, mut health, physics, mut target, ant_entity) in ants.iter_mut() {
        for (mut colony_resources, colony_props, colony_transform, colony_entity) in colonies.iter_mut() {
            let distance = physics.position.distance(colony_transform.translation.truncate());
            
            if distance < colony_props.radius && !resources.resources.is_empty() {
                // Ant is at colony and has resources to deposit
                let deposited_resources = resources.resources.clone();
                let total_deposited: f32 = deposited_resources.values().sum();
                
                for (resource_type, amount) in resources.resources.drain() {
                    *colony_resources.resources.entry(resource_type).or_insert(0.0) += amount;
                }
                resources.current_weight = 0.0;
                
                // Restore energy
                let old_energy = health.energy;
                health.energy = health.max_energy;
                
                info!("Ant {:?} deposited {:.2} total resources at colony {:?} (distance: {:.1})", 
                      ant_entity, total_deposited, colony_entity, distance);
                debug!("Ant {:?} energy restored: {:.1} -> {:.1}", ant_entity, old_energy, health.energy);
                
                // Log deposited resources
                for (resource_type, amount) in deposited_resources {
                    debug!("  - Deposited {:.2} {}", amount, resource_type);
                }
                
                // Clear target
                *target = AntTarget::None;
            } else if distance < colony_props.radius + 5.0 {
                trace!("Ant {:?} approaching colony {:?} (distance: {:.1}, has_resources: {})", 
                       ant_entity, colony_entity, distance, !resources.resources.is_empty());
            }
        }
    }
}

pub struct AntBehaviorPlugin;

impl Plugin for AntBehaviorPlugin {
    fn build(&self, app: &mut App) {
        info!("🔧 Registering AntBehaviorPlugin with systems");
        app.add_systems(Update, (
            ant_movement_system,
            ant_health_system,
            despawn_dead_ants_system,
            cleanup_dead_ants_system,
            ant_ai_system,
            ant_food_interaction_system,
            ant_colony_interaction_system,
        ));
    }
} 