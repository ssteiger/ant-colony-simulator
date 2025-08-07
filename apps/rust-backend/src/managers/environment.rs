use bevy::prelude::*;
use crate::models::*;
use rand::Rng;

// ============================================================================
// ENVIRONMENT MANAGEMENT SYSTEMS
// ============================================================================

/// System to manage food source regeneration
pub fn food_regeneration_system(
    mut food_sources: Query<&mut FoodSourceProperties, With<FoodSource>>,
    simulation_state: Res<SimulationState>,
) {
    debug!("running food_regeneration_system");
    for mut food in food_sources.iter_mut() {
        if food.is_renewable && food.amount < food.max_amount {
            // Regenerate food over time
            food.amount = (food.amount + food.regeneration_rate).min(food.max_amount);
        }
    }
    debug!("food_regeneration_system returning");
}

/// System to manage food source spoilage
pub fn food_spoilage_system(
    mut commands: Commands,
    mut food_sources: Query<(Entity, &mut FoodSourceProperties), With<FoodSource>>,
    simulation_state: Res<SimulationState>,
) {
    debug!("running food_spoilage_system");
    for (entity, mut food) in food_sources.iter_mut() {
        // Apply spoilage
        food.amount = (food.amount - food.spoilage_rate).max(0.0);
        
        // Remove completely spoiled food
        if food.amount <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
    debug!("food_spoilage_system returning");
}

/// System to spawn new food sources
pub fn food_spawning_system(
    mut commands: Commands,
    food_sources: Query<&FoodSourceProperties, With<FoodSource>>,
    simulation_state: Res<SimulationState>,
    world_bounds: Res<WorldBounds>,
) {
    debug!("running food_spawning_system");
    // Spawn new food sources more frequently (every 1000 ticks instead of 5000)
    if simulation_state.current_tick % 1000 == 0 {
        let current_food_count = food_sources.iter().count();
        let max_food_sources = 75; // Increased maximum number of food sources
        
        if current_food_count < max_food_sources {
            spawn_random_food_source(&mut commands, &world_bounds);
        }
    }
    
    // Also spawn a small chance of bonus food every tick for more dynamic spawning
    if simulation_state.current_tick % 100 == 0 {
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < 0.1 { // 10% chance every 100 ticks
            let current_food_count = food_sources.iter().count();
            let max_food_sources = 75;
            
            if current_food_count < max_food_sources {
                spawn_random_food_source(&mut commands, &world_bounds);
            }
        }
    }
    debug!("food_spawning_system returning");
}

/// System to manage weather effects
pub fn weather_system(
    mut ants: Query<(&mut AntHealth, &AntPhysics), With<Ant>>,
    simulation_state: Res<SimulationState>,
) {
    debug!("running weather_system");
    // Simulate weather effects on ants
    for (mut health, physics) in ants.iter_mut() {
        // Rain reduces ant speed and energy
        if simulation_state.current_tick % 1000 < 500 {
            // Rainy period
            health.energy = (health.energy - 0.2).max(0.0);
        }
        
        // Extreme weather can damage ants
        if simulation_state.current_tick % 2000 < 100 {
            // Storm period
            health.health = (health.health - 0.5).max(0.0);
        }
    }
    debug!("weather_system returning");
}

/// System to manage day/night cycle
pub fn day_night_cycle_system(
    mut ants: Query<(&mut AntHealth, &mut AntPhysics), With<Ant>>,
    simulation_state: Res<SimulationState>,
) {
    debug!("running day_night_cycle_system");
    let time_of_day = (simulation_state.current_tick % 24000) as f32 / 24000.0; // 24-hour cycle
    
    for (mut health, mut physics) in ants.iter_mut() {
        // Night time reduces ant activity
        if time_of_day < 0.25 || time_of_day > 0.75 {
            // Night time
            physics.max_speed *= 0.5;
            health.energy = (health.energy - 0.1).max(0.0);
        } else {
            // Day time - restore normal speed
            physics.max_speed = 50.0; // Reset to base speed
        }
    }
    debug!("day_night_cycle_system returning");
}

/// System to manage seasonal effects
pub fn seasonal_system(
    mut food_sources: Query<&mut FoodSourceProperties, With<FoodSource>>,
    simulation_state: Res<SimulationState>,
) {
    debug!("running seasonal_system");
    let season_progress = (simulation_state.current_tick % 100000) as f32 / 100000.0; // Seasonal cycle
    
    for mut food in food_sources.iter_mut() {
        // Winter reduces food regeneration
        if season_progress < 0.25 || season_progress > 0.75 {
            food.regeneration_rate *= 0.3;
        } else {
            // Spring/Summer - normal regeneration
            food.regeneration_rate = 1.0;
        }
    }
    debug!("seasonal_system returning");
}

/// System to manage environmental hazards
pub fn environmental_hazards_system(
    mut commands: Commands,
    mut ants: Query<(Entity, &mut AntHealth, &AntPhysics), With<Ant>>,
    simulation_state: Res<SimulationState>,
) {
    debug!("running environmental_hazards_system");
    for (entity, mut health, physics) in ants.iter_mut() {
        // Random environmental hazards
        if simulation_state.current_tick % 10000 == 0 {
            let mut rng = rand::thread_rng();
            let hazard_chance = rng.gen::<f32>();
            
            if hazard_chance < 0.01 {
                // 1% chance of hazard
                health.health = (health.health - 20.0).max(0.0);
                
                if health.health <= 0.0 {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
    debug!("environmental_hazards_system returning");
}

/// System to manage world boundaries
pub fn world_boundaries_system(
    mut ants: Query<&mut AntPhysics, With<Ant>>,
    world_bounds: Res<WorldBounds>,
) {
    debug!("running world_boundaries_system");
    for mut physics in ants.iter_mut() {
        // Keep ants within world bounds
        physics.position.x = physics.position.x.clamp(world_bounds.min_x, world_bounds.max_x);
        physics.position.y = physics.position.y.clamp(world_bounds.min_y, world_bounds.max_y);
        
        // Bounce off boundaries
        if physics.position.x <= world_bounds.min_x || physics.position.x >= world_bounds.max_x {
            physics.velocity.x *= -0.5;
        }
        if physics.position.y <= world_bounds.min_y || physics.position.y >= world_bounds.max_y {
            physics.velocity.y *= -0.5;
        }
    }
    debug!("world_boundaries_system returning");
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Spawn a random food source in the world
fn spawn_random_food_source(
    commands: &mut Commands,
    world_bounds: &WorldBounds,
) {
    debug!("running spawn_random_food_source");
    let mut rng = rand::thread_rng();
    
    // Random position within centered world bounds
    let x = rng.gen_range(world_bounds.min_x + 50.0..world_bounds.max_x - 50.0);
    let y = rng.gen_range(world_bounds.min_y + 50.0..world_bounds.max_y - 50.0);
    
    // Random food type
    let food_types = vec!["seeds", "sugar", "protein", "fruit"];
    let food_type = food_types[rng.gen_range(0..food_types.len())];
    
    // Random properties
    let max_amount = rng.gen_range(50.0..200.0);
    let regeneration_rate = rng.gen_range(0.1..2.0);
    let nutritional_value = rng.gen_range(10.0..50.0);
    
    commands.spawn((
        FoodSource,
        FoodSourceProperties {
            food_type: food_type.to_string(),
            amount: max_amount,
            max_amount,
            regeneration_rate,
            is_renewable: true,
            nutritional_value,
            spoilage_rate: 0.01,
            discovery_difficulty: rng.gen_range(0.1..1.0),
        },
        Transform::from_translation(Vec3::new(x, y, 0.0)),
    ));
    debug!("spawn_random_food_source returning");
}

/// Plugin for environment management systems
pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        debug!("running EnvironmentPlugin build");
        app.add_systems(Update, (
            food_regeneration_system,
            food_spoilage_system,
            food_spawning_system,
            weather_system,
            day_night_cycle_system,
            seasonal_system,
            environmental_hazards_system,
            world_boundaries_system,
        ));
        debug!("EnvironmentPlugin build returning");
    }
} 