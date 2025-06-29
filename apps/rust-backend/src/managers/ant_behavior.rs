use crate::models::*;
use rand::prelude::*;
use bevy::prelude::*;

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
    ), With<Ant>>,
    time: Res<Time>,
) {
    for (mut physics, health, state, target, mut transform) in ants.iter_mut() {
        if health.health <= 0.0 {
            continue; // Skip dead ants
        }

        let delta_time = time.delta_seconds();
        
        // Calculate target velocity based on state and target
        let target_velocity = match (state, target) {
            (AntState::Wandering, _) => {
                // Random movement for wandering ants
                let mut rng = rand::thread_rng();
                let angle = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
                Vec2::new(angle.cos(), angle.sin()) * physics.max_speed * 0.3
            }
            (AntState::SeekingFood, AntTarget::Food(_)) => {
                // Move towards food target
                Vec2::ZERO // Will be calculated based on target position
            }
            (AntState::CarryingFood, AntTarget::Colony(_)) => {
                // Move towards colony
                Vec2::ZERO // Will be calculated based on target position
            }
            (AntState::Following, AntTarget::Position(pos)) => {
                // Move towards specific position
                let direction = (*pos - physics.position).normalize();
                direction * physics.max_speed
            }
            _ => Vec2::ZERO,
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
    }
}

pub fn ant_health_system(
    mut ants: Query<(&mut AntHealth, &AntPhysics), With<Ant>>,
    simulation_state: Res<SimulationState>,
) {
    for (mut health, _physics) in ants.iter_mut() {
        // Age the ant
        health.age_ticks += 1;
        
        // Energy decay
        health.energy = (health.energy - 1.0).max(0.0);
        
        // Health decay if no energy
        if health.energy <= 0.0 {
            health.health = (health.health - 5.0).max(0.0);
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
    ), With<Ant>>,
    food_sources: Query<&FoodSourceProperties, With<FoodSource>>,
    colonies: Query<&ColonyProperties, With<Colony>>,
    pheromones: Query<&PheromoneProperties, With<PheromoneTrail>>,
) {
    for (mut state, mut target, health, resources, memory, physics) in ants.iter_mut() {
        // Simple AI logic - can be enhanced with Big Brain
        if health.energy < 20.0 && !resources.resources.is_empty() {
            *state = AntState::CarryingFood;
        } else if health.energy > 50.0 && resources.resources.is_empty() {
            *state = AntState::SeekingFood;
        } else {
            *state = AntState::Wandering;
        }
    }
}

pub fn ant_food_interaction_system(
    mut ants: Query<(
        &mut CarriedResources,
        &mut AntHealth,
        &AntPhysics,
        &mut AntTarget,
        &mut AntMemory,
    ), With<Ant>>,
    mut food_sources: Query<(&mut FoodSourceProperties, &Transform), With<FoodSource>>,
) {
    for (mut resources, mut health, physics, mut target, mut memory) in ants.iter_mut() {
        for (mut food, food_transform) in food_sources.iter_mut() {
            let distance = physics.position.distance(food_transform.translation.truncate());
            
            if distance < 10.0 && food.amount > 0.0 {
                // Ant is close enough to collect food
                let collect_amount = (food.amount * 0.1).min(resources.capacity - resources.current_weight);
                
                if collect_amount > 0.0 {
                    // Collect food
                    food.amount -= collect_amount;
                    *resources.resources.entry(food.food_type.clone()).or_insert(0.0) += collect_amount;
                    resources.current_weight += collect_amount;
                    
                    // Update ant state
                    *target = AntTarget::None;
                    
                    // Remember this food source
                    if !memory.known_food_sources.contains(&Entity::PLACEHOLDER) {
                        memory.known_food_sources.push(Entity::PLACEHOLDER);
                    }
                    memory.last_food_source = Some(Entity::PLACEHOLDER);
                }
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
    ), With<Ant>>,
    mut colonies: Query<(&mut ColonyResources, &ColonyProperties, &Transform), With<Colony>>,
) {
    for (mut resources, mut health, physics, mut target) in ants.iter_mut() {
        for (mut colony_resources, colony_props, colony_transform) in colonies.iter_mut() {
            let distance = physics.position.distance(colony_transform.translation.truncate());
            
            if distance < colony_props.radius && !resources.resources.is_empty() {
                // Ant is at colony and has resources to deposit
                for (resource_type, amount) in resources.resources.drain() {
                    *colony_resources.resources.entry(resource_type).or_insert(0.0) += amount;
                }
                resources.current_weight = 0.0;
                
                // Restore energy
                health.energy = health.max_energy;
                
                // Clear target
                *target = AntTarget::None;
            }
        }
    }
}

pub struct AntBehaviorPlugin;

impl Plugin for AntBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            ant_movement_system,
            ant_health_system,
            ant_ai_system,
            ant_food_interaction_system,
            ant_colony_interaction_system,
        ));
    }
} 