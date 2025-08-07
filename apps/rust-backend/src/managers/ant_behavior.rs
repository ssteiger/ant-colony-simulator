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
        &mut AntMemory,
        &AntHealth,
        &AntState,
        &AntTarget,
        &mut Transform,
        Entity,
    ), With<Ant>>,
    world_bounds: Res<WorldBounds>,
    simulation_state: Res<SimulationState>,
    time: Res<Time>,
) {
    for (mut physics, mut memory, health, state, target, mut transform, entity) in ants.iter_mut() {
        if health.health <= 0.0 {
            trace!("Ant {:?} is dead, skipping movement", entity);
            continue; // Skip dead ants
        }

        let delta_time = time.delta_seconds();
        let old_position = physics.position;
        
        // Check if ant is stuck
        let is_stuck = check_if_stuck(&mut physics, &mut memory, simulation_state.current_tick);
        
        // Calculate desired direction based on state and target
        let desired_direction = calculate_desired_direction(
            &physics,
            &memory,
            state,
            target,
            &world_bounds,
            is_stuck,
            entity
        );
        
        // Apply steering behaviors
        let steering_force = calculate_steering_force(
            &mut physics,
            desired_direction,
            &world_bounds,
            delta_time
        );
        
        // Update physics with smooth movement
        update_ant_physics(&mut physics, steering_force, delta_time);
        
        // Update position history
        update_position_history(&mut physics, &mut memory);
        
        // Boundary enforcement
        enforce_world_boundaries(&mut physics, &world_bounds);
        
        // Update transform
        transform.translation.x = physics.position.x;
        transform.translation.y = physics.position.y;
        
        // Smooth rotation based on movement direction
        if physics.velocity.length() > 0.1 {
            let target_rotation = physics.velocity.y.atan2(physics.velocity.x);
            physics.rotation = lerp_angle(physics.rotation, target_rotation, physics.turn_smoothness * delta_time);
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

/// Check if an ant is stuck in the same area
fn check_if_stuck(physics: &mut AntPhysics, memory: &mut AntMemory, current_tick: i64) -> bool {
    // Update position history
    physics.last_positions.push(physics.position);
    if physics.last_positions.len() > 10 {
        physics.last_positions.remove(0);
    }
    
    // Check every 30 ticks
    if current_tick - memory.last_stuck_check > 30 {
        memory.last_stuck_check = current_tick;
        
        if physics.last_positions.len() >= 5 {
            let recent_positions = &physics.last_positions[physics.last_positions.len()-5..];
            let mut total_distance = 0.0;
            
            for i in 1..recent_positions.len() {
                total_distance += recent_positions[i-1].distance(recent_positions[i]);
            }
            
            // If ant moved less than 10 units in 5 position samples, it's stuck
            if total_distance < 10.0 {
                memory.stuck_counter += 1;
                debug!("Ant seems stuck, total distance: {:.2}, stuck_counter: {}", total_distance, memory.stuck_counter);
                return memory.stuck_counter > 2;
            } else {
                memory.stuck_counter = 0;
            }
        }
    }
    
    memory.stuck_counter > 2
}

/// Calculate the desired direction for movement
fn calculate_desired_direction(
    physics: &AntPhysics,
    memory: &AntMemory,
    state: &AntState,
    target: &AntTarget,
    world_bounds: &WorldBounds,
    is_stuck: bool,
    entity: Entity,
) -> Vec2 {
    // If stuck, use escape behavior
    if is_stuck {
        debug!("Ant {:?} is stuck, using escape behavior", entity);
        return calculate_escape_direction(physics, memory, world_bounds);
    }
    
    match (state, target) {
        (AntState::Wandering, _) => {
            calculate_wander_direction(physics, memory)
        }
        (AntState::SeekingFood, AntTarget::Food(_)) | 
        (AntState::CarryingFood, AntTarget::Colony(_)) => {
            // Will be handled by targeted movement system
            calculate_wander_direction(physics, memory)
        }
        (AntState::Following, AntTarget::Position(pos)) => {
            (*pos - physics.position).normalize_or_zero()
        }
        _ => {
            calculate_wander_direction(physics, memory)
        }
    }
}

/// Calculate escape direction when stuck
fn calculate_escape_direction(physics: &AntPhysics, memory: &AntMemory, world_bounds: &WorldBounds) -> Vec2 {
    let mut escape_direction = Vec2::ZERO;
    
    // Avoid recently visited positions
    for visited_pos in &memory.visited_positions {
        let to_visited = *visited_pos - physics.position;
        let distance = to_visited.length();
        if distance < 30.0 && distance > 0.1 {
            let avoidance_strength = (30.0 - distance) / 30.0;
            escape_direction -= to_visited.normalize() * avoidance_strength;
        }
    }
    
    // Avoid boundaries
    let boundary_avoidance = calculate_boundary_avoidance(physics.position, world_bounds);
    escape_direction += boundary_avoidance;
    
    // Add some randomness
    let mut rng = rand::thread_rng();
    let random_angle = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
    let random_direction = Vec2::new(random_angle.cos(), random_angle.sin());
    escape_direction += random_direction * 0.5;
    
    escape_direction.normalize_or_zero()
}

/// Calculate realistic wandering behavior
fn calculate_wander_direction(physics: &AntPhysics, _memory: &AntMemory) -> Vec2 {
    let mut rng = rand::thread_rng();
    
    // Update wander angle with small random changes for smooth movement
    let wander_angle = physics.wander_angle + (rng.gen::<f32>() - 0.5) * physics.wander_change;
    
    // Create a circle in front of the ant for wander target
    let circle_center = physics.position + physics.velocity.normalize_or_zero() * 30.0;
    let wander_target = circle_center + Vec2::new(wander_angle.cos(), wander_angle.sin()) * 15.0;
    
    (wander_target - physics.position).normalize_or_zero()
}

/// Calculate steering forces including obstacle avoidance
fn calculate_steering_force(
    physics: &mut AntPhysics,
    desired_direction: Vec2,
    world_bounds: &WorldBounds,
    delta_time: f32,
) -> Vec2 {
    let mut steering_force = Vec2::ZERO;
    
    // Desired velocity
    let desired_velocity = desired_direction * physics.max_speed;
    
    // Steering = desired - current
    let seek_force = desired_velocity - physics.velocity;
    steering_force += seek_force * 0.5;
    
    // Boundary avoidance
    let boundary_force = calculate_boundary_avoidance(physics.position, world_bounds);
    steering_force += boundary_force * 2.0;
    
    // Update wander angle
    let mut rng = rand::thread_rng();
    physics.wander_angle += (rng.gen::<f32>() - 0.5) * physics.wander_change * delta_time;
    
    steering_force
}

/// Calculate force to avoid world boundaries
fn calculate_boundary_avoidance(position: Vec2, world_bounds: &WorldBounds) -> Vec2 {
    let mut avoidance_force = Vec2::ZERO;
    let buffer = 50.0; // Start avoiding when this close to boundary
    
    // Left boundary
    if position.x < buffer {
        let strength = (buffer - position.x) / buffer;
        avoidance_force.x += strength * 200.0;
    }
    
    // Right boundary
    if position.x > world_bounds.width - buffer {
        let strength = (position.x - (world_bounds.width - buffer)) / buffer;
        avoidance_force.x -= strength * 200.0;
    }
    
    // Bottom boundary
    if position.y < buffer {
        let strength = (buffer - position.y) / buffer;
        avoidance_force.y += strength * 200.0;
    }
    
    // Top boundary
    if position.y > world_bounds.height - buffer {
        let strength = (position.y - (world_bounds.height - buffer)) / buffer;
        avoidance_force.y -= strength * 200.0;
    }
    
    avoidance_force
}

/// Update ant physics with smooth acceleration
fn update_ant_physics(physics: &mut AntPhysics, steering_force: Vec2, delta_time: f32) {
    // Apply steering force with momentum consideration
    let acceleration = steering_force * physics.acceleration * delta_time;
    physics.velocity += acceleration;
    
    // Apply momentum for more realistic movement
    let momentum = physics.momentum;
    physics.velocity *= momentum;
    
    // Clamp velocity to max speed
    if physics.velocity.length() > physics.max_speed {
        physics.velocity = physics.velocity.normalize() * physics.max_speed;
    }
    
    // Update position
    physics.position += physics.velocity * delta_time;
}

/// Update position history for path tracking
fn update_position_history(physics: &mut AntPhysics, memory: &mut AntMemory) {
    // Add current position to path history
    memory.path_history.push(physics.position);
    if memory.path_history.len() > 20 {
        memory.path_history.remove(0);
    }
    
    // Update visited positions (with some spacing to avoid too many points)
    if memory.visited_positions.is_empty() || 
       memory.visited_positions.last().unwrap().distance(physics.position) > 15.0 {
        memory.visited_positions.push(physics.position);
        if memory.visited_positions.len() > 50 {
            memory.visited_positions.remove(0);
        }
    }
}

/// Enforce world boundaries by keeping ants inside
fn enforce_world_boundaries(physics: &mut AntPhysics, world_bounds: &WorldBounds) {
    // Hard boundary enforcement
    if physics.position.x < 0.0 {
        physics.position.x = 0.0;
        physics.velocity.x = physics.velocity.x.abs(); // Bounce off
    }
    if physics.position.x > world_bounds.width {
        physics.position.x = world_bounds.width;
        physics.velocity.x = -physics.velocity.x.abs(); // Bounce off
    }
    if physics.position.y < 0.0 {
        physics.position.y = 0.0;
        physics.velocity.y = physics.velocity.y.abs(); // Bounce off
    }
    if physics.position.y > world_bounds.height {
        physics.position.y = world_bounds.height;
        physics.velocity.y = -physics.velocity.y.abs(); // Bounce off
    }
}

/// Linear interpolation between two angles
fn lerp_angle(from: f32, to: f32, t: f32) -> f32 {
    let diff = (to - from + std::f32::consts::PI) % (2.0 * std::f32::consts::PI) - std::f32::consts::PI;
    from + diff * t
}

// Separate system to handle targeted movement towards food and colonies
pub fn ant_targeted_movement_system(
    mut ants: Query<(
        &mut AntPhysics,
        &mut AntMemory,
        &AntState,
        &AntTarget,
        Entity,
    ), With<Ant>>,
    food_sources: Query<(&FoodSourceProperties, &Transform, Entity), With<FoodSource>>,
    colonies: Query<(&ColonyProperties, &Transform, Entity), With<Colony>>,
    world_bounds: Res<WorldBounds>,
    time: Res<Time>,
) {
    for (mut physics, memory, state, target, entity) in ants.iter_mut() {
        let delta_time = time.delta_seconds();
        
        // Calculate targeted direction with improved pathfinding
        let target_direction = match (state, target) {
            (AntState::SeekingFood, AntTarget::Food(food_entity)) => {
                // Move towards food target with obstacle avoidance
                if let Ok((_food_props, food_transform, _)) = food_sources.get(*food_entity) {
                    let food_pos = food_transform.translation.truncate();
                    let direction = calculate_smart_direction(
                        physics.position,
                        food_pos,
                        &memory,
                        &world_bounds
                    );
                    trace!("Ant {:?} seeking food at ({:.2}, {:.2})", 
                           entity, food_pos.x, food_pos.y);
                    Some(direction)
                } else {
                    trace!("Ant {:?} seeking food but target entity {:?} not found", entity, food_entity);
                    None
                }
            }
            (AntState::CarryingFood, AntTarget::Colony(colony_entity)) => {
                // Move towards colony with obstacle avoidance
                if let Ok((_colony_props, colony_transform, _)) = colonies.get(*colony_entity) {
                    let colony_pos = colony_transform.translation.truncate();
                    let direction = calculate_smart_direction(
                        physics.position,
                        colony_pos,
                        &memory,
                        &world_bounds
                    );
                    trace!("Ant {:?} carrying food to colony at ({:.2}, {:.2})", 
                           entity, colony_pos.x, colony_pos.y);
                    Some(direction)
                } else {
                    trace!("Ant {:?} carrying food but colony entity {:?} not found", entity, colony_entity);
                    None
                }
            }
            _ => None,
        };

        // Apply targeted movement with improved steering
        if let Some(direction) = target_direction {
            // Use the desired direction for the main movement system
            physics.desired_direction = direction;
            
            // Apply gentle steering towards target
            let desired_velocity = direction * physics.max_speed * 0.8; // Slightly slower when targeting
            let steering_force = (desired_velocity - physics.velocity) * 0.3; // Gentle steering
            
            // Apply boundary avoidance
            let boundary_force = calculate_boundary_avoidance(physics.position, &world_bounds);
            let total_force = steering_force + boundary_force * 1.5;
            
            // Update velocity with smooth acceleration
            let acceleration = total_force * physics.acceleration * delta_time;
            physics.velocity += acceleration;
            
            // Apply momentum
            let momentum = physics.momentum;
            physics.velocity *= momentum;
            
            // Clamp velocity to max speed
            if physics.velocity.length() > physics.max_speed {
                physics.velocity = physics.velocity.normalize() * physics.max_speed;
            }
        }
    }
}

/// Calculate smart direction towards target avoiding obstacles and visited areas
fn calculate_smart_direction(
    current_pos: Vec2,
    target_pos: Vec2,
    memory: &AntMemory,
    world_bounds: &WorldBounds,
) -> Vec2 {
    let mut direction = (target_pos - current_pos).normalize_or_zero();
    
    // Avoid recently visited areas when pathfinding
    for visited_pos in &memory.visited_positions {
        let to_visited = *visited_pos - current_pos;
        let distance = to_visited.length();
        
        // If we're close to a visited position and it's in our path, slightly avoid it
        if distance < 25.0 && distance > 0.1 {
            let dot_product = direction.dot(to_visited.normalize());
            if dot_product > 0.5 { // If visited position is in our general direction
                let avoidance_strength = ((25.0 - distance) / 25.0) * 0.3;
                let perpendicular = Vec2::new(-to_visited.y, to_visited.x).normalize();
                direction += perpendicular * avoidance_strength;
            }
        }
    }
    
    // Avoid boundaries in pathfinding
    let boundary_avoidance = calculate_boundary_avoidance(current_pos, world_bounds) * 0.001;
    direction += boundary_avoidance;
    
    direction.normalize_or_zero()
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
                
                // For now, just set a random colony target - will be refined by separate system
                *target = AntTarget::None;
            }
        } else if health.energy > 50.0 && resources.resources.is_empty() {
            *state = AntState::SeekingFood;
            if old_state != *state {
                info!("Ant {:?} switching to SeekingFood (energy: {:.1})", entity, health.energy);
                
                // For now, just set a random food target - will be refined by separate system
                *target = AntTarget::None;
            }
        } else {
            *state = AntState::Wandering;
            if old_state != *state {
                debug!("Ant {:?} switching to Wandering (energy: {:.1}, resources: {:.1})", 
                       entity, health.energy, resources.current_weight);
                *target = AntTarget::None;
            }
        }

        // Log AI decision making
        trace!("Ant {:?} AI decision: state={:?}, energy={:.1}, resources={:.1}, known_food_sources={}", 
               entity, *state, health.energy, resources.current_weight, memory.known_food_sources.len());
    }
}

// Separate system to set targets for ants
pub fn ant_target_setting_system(
    mut ants: Query<(
        &mut AntTarget,
        &AntState,
        &AntPhysics,
        Entity,
    ), With<Ant>>,
    food_sources: Query<(&FoodSourceProperties, &Transform, Entity), With<FoodSource>>,
    colonies: Query<(&ColonyProperties, &Transform, Entity), With<Colony>>,
) {
    for (mut target, state, physics, entity) in ants.iter_mut() {
        match state {
            AntState::SeekingFood => {
                // Find nearest food source
                let mut nearest_food = None;
                let mut nearest_distance = f32::INFINITY;
                
                for (food_props, food_transform, food_entity) in food_sources.iter() {
                    if food_props.amount > 0.0 { // Only target food sources with food
                        let distance = physics.position.distance(food_transform.translation.truncate());
                        if distance < nearest_distance {
                            nearest_distance = distance;
                            nearest_food = Some(food_entity);
                        }
                    }
                }
                
                if let Some(food_entity) = nearest_food {
                    *target = AntTarget::Food(food_entity);
                    debug!("Ant {:?} targeting food source {:?} at distance {:.1}", entity, food_entity, nearest_distance);
                } else {
                    debug!("Ant {:?} can't find any food sources, will wander", entity);
                    *target = AntTarget::None;
                }
            }
            AntState::CarryingFood => {
                // Find nearest colony to return to
                let mut nearest_colony = None;
                let mut nearest_distance = f32::INFINITY;
                
                for (_colony_props, colony_transform, colony_entity) in colonies.iter() {
                    let distance = physics.position.distance(colony_transform.translation.truncate());
                    if distance < nearest_distance {
                        nearest_distance = distance;
                        nearest_colony = Some(colony_entity);
                    }
                }
                
                if let Some(colony_entity) = nearest_colony {
                    *target = AntTarget::Colony(colony_entity);
                    debug!("Ant {:?} targeting colony {:?} at distance {:.1}", entity, colony_entity, nearest_distance);
                } else {
                    warn!("Ant {:?} can't find any colony to return to!", entity);
                    *target = AntTarget::None;
                }
            }
            _ => {
                // Keep current target for other states
            }
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
        Entity,
    ), With<Ant>>,
    mut food_sources: Query<(&mut FoodSourceProperties, &Transform, Entity), With<FoodSource>>,
) {
    for (mut resources, _health, physics, mut target, mut memory, ant_entity) in ants.iter_mut() {
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
            ant_targeted_movement_system,
            ant_health_system,
            despawn_dead_ants_system,
            cleanup_dead_ants_system,
            ant_ai_system,
            ant_target_setting_system,
            ant_food_interaction_system,
            ant_colony_interaction_system,
        ));
    }
} 