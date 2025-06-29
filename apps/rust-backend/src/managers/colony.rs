use crate::models::*;
use rand::prelude::*;
use bevy::prelude::*;
use std::collections::HashMap;

// ============================================================================
// COLONY MANAGEMENT SYSTEMS
// ============================================================================

/// System to manage colony population and spawning
pub fn colony_spawning_system(
    mut commands: Commands,
    mut colonies: Query<(
        &mut ColonyProperties,
        &mut ColonyResources,
        &ColonyNest,
        &Transform,
    ), With<Colony>>,
    ants: Query<&Ant, With<Ant>>,
    simulation_state: Res<SimulationState>,
) {
    for (mut colony_props, mut resources, nest, transform) in colonies.iter_mut() {
        // Count ants in this colony
        let ant_count = ants.iter().count() as i32;
        colony_props.population = ant_count;

        // Check if colony can spawn new ants
        if colony_props.population < colony_props.max_population {
            let can_spawn = has_sufficient_resources(&resources.resources, &nest.upgrade_cost);
            
            if can_spawn && simulation_state.current_tick % 100 == 0 {
                // Spawn a new ant
                spawn_ant(&mut commands, &colony_props, transform.translation.truncate());
                
                // Consume resources for spawning
                consume_spawning_resources(&mut resources.resources, &nest.upgrade_cost);
            }
        }
    }
}

/// System to manage colony resource consumption
pub fn colony_resource_consumption_system(
    mut colonies: Query<(&mut ColonyResources, &ColonyProperties), With<Colony>>,
    simulation_state: Res<SimulationState>,
) {
    for (mut resources, colony_props) in colonies.iter_mut() {
        // Colonies consume resources over time
        if simulation_state.current_tick % 1000 == 0 {
            let consumption_rate = colony_props.population as f32 * 0.1;
            
            for (resource_type, amount) in resources.resources.iter_mut() {
                *amount = (*amount - consumption_rate).max(0.0);
            }
        }
    }
}

/// System to manage colony upgrades
pub fn colony_upgrade_system(
    mut colonies: Query<(
        &mut ColonyNest,
        &mut ColonyProperties,
        &mut ColonyResources,
    ), With<Colony>>,
) {
    for (mut nest, mut colony_props, mut resources) in colonies.iter_mut() {
        // Check if colony can upgrade
        if nest.level < nest.max_level {
            let can_upgrade = has_sufficient_resources(&resources.resources, &nest.upgrade_cost);
            
            if can_upgrade {
                // Perform upgrade
                nest.level += 1;
                colony_props.max_population += 10;
                colony_props.territory_radius += 20.0;
                
                // Consume upgrade resources
                consume_spawning_resources(&mut resources.resources, &nest.upgrade_cost);
                
                // Increase upgrade cost for next level
                for (_, cost) in nest.upgrade_cost.iter_mut() {
                    *cost *= 1.5;
                }
            }
        }
    }
}

/// System to manage colony territory and aggression
pub fn colony_territory_system(
    mut commands: Commands,
    colonies: Query<(
        &ColonyProperties,
        &Transform,
    ), With<Colony>>,
    mut ants: Query<(
        &AntPhysics,
        &AntHealth,
        &mut AntState,
    ), With<Ant>>,
) {
    // Check for inter-colony conflicts
    let colony_positions: Vec<(Entity, Vec2, f32, f32)> = colonies
        .iter()
        .map(|(props, transform)| {
            (
                Entity::PLACEHOLDER, // Would need actual entity
                transform.translation.truncate(),
                props.territory_radius,
                props.aggression_level,
            )
        })
        .collect();

    for (ant_physics, ant_health, mut ant_state) in ants.iter_mut() {
        // Check if ant is in enemy territory
        for (_, colony_pos, territory_radius, aggression) in &colony_positions {
            let distance = ant_physics.position.distance(*colony_pos);
            
            if distance < *territory_radius && *aggression > 0.5 {
                // Ant is in enemy territory and colony is aggressive
                if ant_health.health < 50.0 {
                    // Weak ant gets killed
                    *ant_state = AntState::Dead;
                }
            }
        }
    }
}

/// System to manage colony statistics
pub fn colony_stats_system(
    mut stats: ResMut<SimulationStats>,
    colonies: Query<&ColonyProperties, With<Colony>>,
    ants: Query<&Ant, With<Ant>>,
) {
    stats.active_colonies = colonies.iter().count() as i32;
    stats.total_ants = ants.iter().count() as i32;
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Check if colony has sufficient resources for spawning/upgrading
fn has_sufficient_resources(
    resources: &HashMap<String, f32>,
    required: &HashMap<String, f32>,
) -> bool {
    for (resource_type, required_amount) in required {
        let available = resources.get(resource_type).unwrap_or(&0.0);
        if available < required_amount {
            return false;
        }
    }
    true
}

/// Consume resources for spawning/upgrading
fn consume_spawning_resources(
    resources: &mut HashMap<String, f32>,
    cost: &HashMap<String, f32>,
) {
    for (resource_type, cost_amount) in cost {
        if let Some(available) = resources.get_mut(resource_type) {
            *available = (*available - cost_amount).max(0.0);
        }
    }
}

/// Spawn a new ant for a colony
fn spawn_ant(
    commands: &mut Commands,
    colony_props: &ColonyProperties,
    colony_position: Vec2,
) {
    // Generate random position near colony center
    let mut rng = rand::thread_rng();
    let angle = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
    let distance = rng.gen::<f32>() * colony_props.radius;
    let spawn_position = colony_position + Vec2::new(angle.cos(), angle.sin()) * distance;

    commands.spawn((
        Ant,
        AntPhysics {
            position: spawn_position,
            velocity: Vec2::ZERO,
            max_speed: 50.0,
            acceleration: 100.0,
            rotation: 0.0,
            rotation_speed: 2.0,
        },
        AntHealth {
            health: 100.0,
            max_health: 100.0,
            energy: 100.0,
            max_energy: 100.0,
            age_ticks: 0,
            lifespan_ticks: 10000,
        },
        AntState::Wandering,
        CarriedResources {
            resources: HashMap::new(),
            capacity: 10.0,
            current_weight: 0.0,
        },
        AntTarget::None,
        AntMemory {
            known_food_sources: Vec::new(),
            known_colonies: Vec::new(),
            last_food_source: None,
            last_action_tick: 0,
            pheromone_sensitivity: 1.0,
        },
        AntType {
            name: "Worker".to_string(),
            role: "worker".to_string(),
            base_speed: 50.0,
            base_strength: 10.0,
            base_health: 100.0,
            carrying_capacity: 10.0,
            color_hue: colony_props.color_hue,
            special_abilities: Vec::new(),
        },
        Transform::from_translation(Vec3::new(spawn_position.x, spawn_position.y, 0.0)),
    ));
}

/// Plugin for colony management systems
pub struct ColonyPlugin;

impl Plugin for ColonyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            colony_spawning_system,
            colony_resource_consumption_system,
            colony_upgrade_system,
            colony_territory_system,
            colony_stats_system,
        ));
    }
} 