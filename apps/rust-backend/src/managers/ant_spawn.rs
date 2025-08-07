use crate::models::*;
use crate::managers::ant_behavior::*;
use rand::prelude::*;
use bevy::prelude::*;
use big_brain::prelude::*;
use tracing::{debug, info};

/// Function to spawn an ant with big-brain AI
pub fn spawn_ant_with_big_brain(
    commands: &mut Commands, 
    position: Vec2,
    colony_id: Option<Entity>
) -> Entity {
    // Determine ant type (90% workers, 10% scouts)
    let mut rng = rand::thread_rng();
    let role = if rng.gen::<f32>() < 0.1 { "scout" } else { "worker" };
    
    commands.spawn((
        // Core ant components
        Ant,
        AntPhysics {
            position,
            velocity: Vec2::ZERO,
            max_speed: 50.0,
            acceleration: 100.0,
            rotation: 0.0,
            rotation_speed: 2.0,
            desired_direction: Vec2::ZERO,
            momentum: 0.95,
            last_positions: Vec::new(),
            turn_smoothness: 3.0,
            wander_angle: rand::thread_rng().gen::<f32>() * 2.0 * std::f32::consts::PI,
            wander_change: 0.5,
            obstacle_avoidance_force: Vec2::ZERO,
        },
        AntHealth {
            health: 100.0,
            max_health: 100.0,
            age_ticks: 0,
            lifespan_ticks: 10000,
        },
        AntState::Wandering,
        CarriedResources {
            resources: std::collections::HashMap::new(),
            capacity: 25.0,
            current_weight: 0.0,
        },
        AntType {
            name: if role == "scout" { "Scout".to_string() } else { "Worker".to_string() },
            role: role.to_string(),
            base_speed: 50.0,
            base_strength: 10.0,
            base_health: 100.0,
            carrying_capacity: 25.0,
            color_hue: 0.0, // Will be set by colony if provided
            special_abilities: Vec::new(),
        },
        AntMemory {
            known_food_sources: Vec::new(),
            known_colonies: if let Some(id) = colony_id { vec![id] } else { Vec::new() },
            last_food_source: None,
            last_action_tick: 0,
            pheromone_sensitivity: 1.0,
            visited_positions: Vec::new(),
            last_stuck_check: 0,
            stuck_counter: 0,
            exploration_radius: 100.0,
            path_history: Vec::new(),
        },
        AntTarget::None,
        Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
        
        // Big-brain AI system - Role-based behavior driven by pheromones
        Thinker::build()
            .picker(Highest) // Choose action with highest utility score
            
            // Critical priorities (survival)
            .when(StuckScorer, EscapeAction)
            
            // High priority (food management for workers)
            .when(CarryingFoodScorer, ReturnToColonyAction)
            .when(NearColonyScorer, DepositFoodAction)
            
            // Medium priority (role-specific behaviors)
            .when(NearFoodScorer, CollectFoodAction)         // Any ant near food will collect it
            .when(PheromoneStrengthScorer, SeekFoodAction)   // Workers follow pheromone trails to food
            .when(ScoutScorer, MarkFoodSourceAction)         // Scouts mark discovered food
            .when(ScoutScorer, SeekFoodAction)               // Scouts actively seek new food sources
            
            // Low priority (basic food seeking and exploration)
            .when(ExplorationUrgeScorer, SeekFoodAction)     // All ants occasionally seek food
            .otherwise(WanderAction),
    )).id()
}

/// System to handle ant aging (no energy system)
pub fn ant_health_system(
    mut ants: Query<(&mut AntHealth, &AntPhysics, Entity), With<Ant>>,
) {
    for (mut health, _physics, entity) in ants.iter_mut() {
        // Age the ant
        health.age_ticks += 1;
        
        // Health remains stable unless damaged
        // (Ants work based on pheromones, not energy)
        
        if health.age_ticks % 1000 == 0 {
            debug!("Ant {:?} age: {} ticks, health: {:.1}", entity, health.age_ticks, health.health);
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

/// Basic movement system that works with big-brain targets
pub fn basic_ant_movement_system(
    mut ants: Query<(
        &mut AntPhysics,
        &AntTarget,
        &mut Transform,
        Entity,
    ), With<Ant>>,
    food_sources: Query<&Transform, (With<FoodSource>, Without<Ant>)>,
    colonies: Query<&Transform, (With<Colony>, Without<Ant>)>,
    time: Res<Time>,
) {
    for (mut physics, target, mut transform, entity) in ants.iter_mut() {
        let delta_time = time.delta_seconds();
        
        // Calculate desired direction based on target
        let desired_direction = match target {
            AntTarget::Food(food_entity) => {
                if let Ok(food_transform) = food_sources.get(*food_entity) {
                    let food_pos = food_transform.translation.truncate();
                    (food_pos - physics.position).normalize_or_zero()
                } else {
                    calculate_wander_direction(&mut physics)
                }
            }
            AntTarget::Colony(colony_entity) => {
                if let Ok(colony_transform) = colonies.get(*colony_entity) {
                    let colony_pos = colony_transform.translation.truncate();
                    (colony_pos - physics.position).normalize_or_zero()
                } else {
                    calculate_wander_direction(&mut physics)
                }
            }
            AntTarget::Position(pos) => {
                (*pos - physics.position).normalize_or_zero()
            }
            AntTarget::None => {
                calculate_wander_direction(&mut physics)
            }
        };
        
        // Apply simple steering
        let desired_velocity = desired_direction * physics.max_speed;
        let steering = (desired_velocity - physics.velocity) * 0.1;
        physics.velocity += steering;
        
        // Apply momentum
        let momentum = physics.momentum;
        physics.velocity *= momentum;
        
        // Clamp velocity
        if physics.velocity.length() > physics.max_speed {
            physics.velocity = physics.velocity.normalize() * physics.max_speed;
        }
        
        // Update position
        let velocity = physics.velocity;
        physics.position += velocity * delta_time;
        
        // Update transform
        transform.translation.x = physics.position.x;
        transform.translation.y = physics.position.y;
        
        // Update rotation
        if physics.velocity.length() > 0.1 {
            physics.rotation = physics.velocity.y.atan2(physics.velocity.x);
            transform.rotation = Quat::from_rotation_z(physics.rotation);
        }
        
        // Update position history for pheromone trails and stuck detection
        let current_position = physics.position;
        physics.last_positions.push(current_position);
        if physics.last_positions.len() > 10 {
            physics.last_positions.remove(0);
        }
    }
}

/// Calculate realistic wandering behavior
fn calculate_wander_direction(physics: &mut AntPhysics) -> Vec2 {
    let mut rng = rand::thread_rng();
    
    // Update wander angle with small random changes for smooth movement
    physics.wander_angle += (rng.gen::<f32>() - 0.5) * physics.wander_change;
    
    // Create a circle in front of the ant for wander target
    let circle_center = physics.position + physics.velocity.normalize_or_zero() * 30.0;
    let wander_target = circle_center + Vec2::new(physics.wander_angle.cos(), physics.wander_angle.sin()) * 15.0;
    
    (wander_target - physics.position).normalize_or_zero()
}