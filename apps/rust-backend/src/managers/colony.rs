use crate::models::*;
use rand::prelude::*;
use bevy::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info, warn, error, instrument};

// ============================================================================
// COLONY MANAGEMENT SYSTEMS
// ============================================================================

/// System to manage colony population and spawning
#[instrument(skip(commands, colonies, ants, simulation_state))]
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
    let mut total_spawned = 0;
    let mut total_ants = 0;

    for (mut colony_props, mut resources, nest, transform) in colonies.iter_mut() {
        // Count ants in this colony
        let ant_count = ants.iter().count() as i32;
        let old_population = colony_props.population;
        colony_props.population = ant_count;
        total_ants += ant_count;

        if old_population != ant_count {
            debug!(
                "Colony population updated: {} -> {} (max: {})",
                old_population, ant_count, colony_props.max_population
            );
        }

        // Check if colony can spawn new ants
        if colony_props.population < colony_props.max_population {
            let can_spawn = has_sufficient_resources(&resources.resources, &nest.upgrade_cost);
            
            debug!(
                "Colony spawning check: population={}, max_population={}, can_spawn={}, tick={}",
                colony_props.population, colony_props.max_population, can_spawn, simulation_state.current_tick
            );
            
            if can_spawn && simulation_state.current_tick % 100 == 0 {
                // Spawn a new ant
                let ant_entity = spawn_ant(&mut commands, &colony_props, transform.translation.truncate());
                total_spawned += 1;
                
                debug!(
                    "Spawned ant {:?} for colony at ({:.2}, {:.2}), new population: {}",
                    ant_entity, transform.translation.x, transform.translation.y, colony_props.population + 1
                );
                
                // Consume resources for spawning
                let old_resources = resources.resources.clone();
                consume_spawning_resources(&mut resources.resources, &nest.upgrade_cost);
                
                debug!(
                    "Consumed spawning resources: {:?} -> {:?}",
                    old_resources, resources.resources
                );
            }
        } else {
            debug!(
                "Colony at max population: {} >= {}",
                colony_props.population, colony_props.max_population
            );
        }
    }

    if total_spawned > 0 {
        info!(
            "Colony spawning complete: {} ants spawned, total ants: {}, tick: {}",
            total_spawned, total_ants, simulation_state.current_tick
        );
    }
}

/// System to manage colony resource consumption
#[instrument(skip(colonies, simulation_state))]
pub fn colony_resource_consumption_system(
    mut colonies: Query<(&mut ColonyResources, &ColonyProperties), With<Colony>>,
    simulation_state: Res<SimulationState>,
) {
    let mut colonies_consuming = 0;

    for (mut resources, colony_props) in colonies.iter_mut() {
        // Colonies consume resources over time
        if simulation_state.current_tick % 1000 == 0 {
            let consumption_rate = colony_props.population as f32 * 0.1;
            let old_resources = resources.resources.clone();
            
            for (resource_type, amount) in resources.resources.iter_mut() {
                let old_amount = *amount;
                *amount = (*amount - consumption_rate).max(0.0);
                
                if *amount != old_amount {
                    debug!(
                        "Colony consumed {}: {:.2} -> {:.2} (rate: {:.2})",
                        resource_type, old_amount, *amount, consumption_rate
                    );
                }
            }
            
            colonies_consuming += 1;
            debug!(
                "Colony resource consumption: population={}, rate={:.2}, resources: {:?} -> {:?}",
                colony_props.population, consumption_rate, old_resources, resources.resources
            );
        }
    }

    if colonies_consuming > 0 {
        debug!(
            "Resource consumption: {} colonies consumed resources, tick: {}",
            colonies_consuming, simulation_state.current_tick
        );
    }
}

/// System to manage colony upgrades
#[instrument(skip(colonies))]
pub fn colony_upgrade_system(
    mut colonies: Query<(
        &mut ColonyNest,
        &mut ColonyProperties,
        &mut ColonyResources,
    ), With<Colony>>,
) {
    let mut colonies_upgraded = 0;

    for (mut nest, mut colony_props, mut resources) in colonies.iter_mut() {
        // Check if colony can upgrade
        if nest.level < nest.max_level {
            let can_upgrade = has_sufficient_resources(&resources.resources, &nest.upgrade_cost);
            
            debug!(
                "Colony upgrade check: level={}, max_level={}, can_upgrade={}, cost={:?}",
                nest.level, nest.max_level, can_upgrade, nest.upgrade_cost
            );
            
            if can_upgrade {
                // Perform upgrade
                let old_level = nest.level;
                let old_max_population = colony_props.max_population;
                let old_territory_radius = colony_props.territory_radius;
                
                nest.level += 1;
                colony_props.max_population += 10;
                colony_props.territory_radius += 20.0;
                
                warn!(
                    "Colony upgraded: level {} -> {}, max_population {} -> {}, territory_radius {:.2} -> {:.2}",
                    old_level, nest.level, old_max_population, colony_props.max_population,
                    old_territory_radius, colony_props.territory_radius
                );
                
                // Consume upgrade resources
                let old_resources = resources.resources.clone();
                consume_spawning_resources(&mut resources.resources, &nest.upgrade_cost);
                
                debug!(
                    "Upgrade resources consumed: {:?} -> {:?}",
                    old_resources, resources.resources
                );
                
                // Increase upgrade cost for next level
                for (resource_type, cost) in nest.upgrade_cost.iter_mut() {
                    let old_cost = *cost;
                    *cost *= 1.5;
                    debug!(
                        "Upgrade cost increased for {}: {:.2} -> {:.2}",
                        resource_type, old_cost, *cost
                    );
                }
                
                colonies_upgraded += 1;
            }
        } else {
            debug!(
                "Colony at max level: {} >= {}",
                nest.level, nest.max_level
            );
        }
    }

    if colonies_upgraded > 0 {
        info!("{} colonies upgraded", colonies_upgraded);
    }
}

/// System to manage colony territory and aggression
#[instrument(skip(commands, colonies, ants))]
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
    let mut ants_killed = 0;
    let mut territory_conflicts = 0;

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

    debug!(
        "Territory check: {} colonies, {} ants",
        colony_positions.len(), ants.iter().count()
    );

    for (ant_physics, ant_health, mut ant_state) in ants.iter_mut() {
        // Check if ant is in enemy territory
        for (colony_entity, colony_pos, territory_radius, aggression) in &colony_positions {
            let distance = ant_physics.position.distance(*colony_pos);
            
            if distance < *territory_radius {
                territory_conflicts += 1;
                debug!(
                    "Ant at ({:.2}, {:.2}) in territory of colony {:?} (distance: {:.2}, radius: {:.2})",
                    ant_physics.position.x, ant_physics.position.y, colony_entity, distance, territory_radius
                );
                
                if *aggression > 0.5 {
                    debug!(
                        "Colony {:?} is aggressive (level: {:.2})",
                        colony_entity, aggression
                    );
                    
                    if ant_health.health < 50.0 {
                        // Weak ant gets killed
                        *ant_state = AntState::Dead;
                        ants_killed += 1;
                        
                        warn!(
                            "Ant killed in territory conflict: health={:.2}, position=({:.2}, {:.2})",
                            ant_health.health, ant_physics.position.x, ant_physics.position.y
                        );
                    } else {
                        debug!(
                            "Ant survived territory conflict: health={:.2}",
                            ant_health.health
                        );
                    }
                }
            }
        }
    }

    if territory_conflicts > 0 || ants_killed > 0 {
        debug!(
            "Territory conflicts: {} ants in enemy territory, {} killed",
            territory_conflicts, ants_killed
        );
    }
}

/// System to manage colony statistics
#[instrument(skip(stats, colonies, ants))]
pub fn colony_stats_system(
    mut stats: ResMut<SimulationStats>,
    colonies: Query<&ColonyProperties, With<Colony>>,
    ants: Query<&Ant, With<Ant>>,
) {
    let old_active_colonies = stats.active_colonies;
    let old_total_ants = stats.total_ants;
    
    stats.active_colonies = colonies.iter().count() as i32;
    stats.total_ants = ants.iter().count() as i32;
    
    if stats.active_colonies != old_active_colonies || stats.total_ants != old_total_ants {
        debug!(
            "Colony stats updated: colonies {} -> {}, ants {} -> {}",
            old_active_colonies, stats.active_colonies, old_total_ants, stats.total_ants
        );
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Check if colony has sufficient resources for spawning/upgrading
#[instrument(skip(resources, required))]
fn has_sufficient_resources(
    resources: &HashMap<String, f32>,
    required: &HashMap<String, f32>,
) -> bool {
    for (resource_type, required_amount) in required {
        let available = resources.get(resource_type).unwrap_or(&0.0);
        if available < required_amount {
            debug!(
                "Insufficient resources: {} (required: {:.2}, available: {:.2})",
                resource_type, required_amount, available
            );
            return false;
        }
    }
    debug!("Sufficient resources available: {:?}", required);
    true
}

/// Consume resources for spawning/upgrading
#[instrument(skip(resources, cost))]
fn consume_spawning_resources(
    resources: &mut HashMap<String, f32>,
    cost: &HashMap<String, f32>,
) {
    for (resource_type, cost_amount) in cost {
        if let Some(available) = resources.get_mut(resource_type) {
            let old_amount = *available;
            *available = (*available - cost_amount).max(0.0);
            
            debug!(
                "Consumed resource {}: {:.2} -> {:.2} (cost: {:.2})",
                resource_type, old_amount, *available, cost_amount
            );
        } else {
            warn!(
                "Attempted to consume non-existent resource: {}",
                resource_type
            );
        }
    }
}

/// Spawn a new ant for a colony
#[instrument(skip(commands, colony_props))]
fn spawn_ant(
    commands: &mut Commands,
    colony_props: &ColonyProperties,
    colony_position: Vec2,
) -> Entity {
    // Generate random position near colony center
    let mut rng = rand::thread_rng();
    let angle = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
    let distance = rng.gen::<f32>() * colony_props.radius;
    let spawn_position = colony_position + Vec2::new(angle.cos(), angle.sin()) * distance;

    debug!(
        "Spawning ant: colony_pos=({:.2}, {:.2}), spawn_pos=({:.2}, {:.2}), angle={:.2}, distance={:.2}",
        colony_position.x, colony_position.y, spawn_position.x, spawn_position.y, angle, distance
    );

    let ant_entity = commands.spawn((
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
    )).id();

    debug!(
        "Ant {:?} spawned successfully with type: Worker, color_hue: {:.2}",
        ant_entity, colony_props.color_hue
    );

    ant_entity
}

/// Plugin for colony management systems
pub struct ColonyPlugin;

impl Plugin for ColonyPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing ColonyPlugin");
        app.add_systems(Update, (
            colony_spawning_system,
            colony_resource_consumption_system,
            colony_upgrade_system,
            colony_territory_system,
            colony_stats_system,
        ));
        debug!("ColonyPlugin systems registered");
    }
} 