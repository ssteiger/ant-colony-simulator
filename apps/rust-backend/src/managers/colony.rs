use crate::models::*;
use rand::prelude::*;
use bevy::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info, warn, instrument};

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
    debug!("running colony_spawning_system");
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
            
            // Spawn ants using configurable frequency and batch size
            if can_spawn && simulation_state.current_tick % simulation_state.spawn_tick_interval == 0 {
                // Calculate how many ants we can spawn this cycle
                let ants_needed = colony_props.max_population - colony_props.population;
                let batch_size = ants_needed.min(simulation_state.max_spawn_batch_size);
                
                // Check if we have enough resources for the batch
                let mut ants_to_spawn = 0;
                for _ in 0..batch_size {
                    if has_sufficient_resources(&resources.resources, &nest.upgrade_cost) {
                        ants_to_spawn += 1;
                        // Pre-consume resources to check next ant
                        consume_spawning_resources(&mut resources.resources, &nest.upgrade_cost);
                    } else {
                        break;
                    }
                }
                
                // Restore resources (we'll consume them properly when spawning)
                for _ in 0..ants_to_spawn {
                    for (resource_type, cost_amount) in &nest.upgrade_cost {
                        if let Some(available) = resources.resources.get_mut(resource_type) {
                            *available += cost_amount;
                        }
                    }
                }
                
                if ants_to_spawn > 0 {
                    // Spawn ants with Big Brain AI
                    let spawned_ants = spawn_ant_batch_with_ai(&mut commands, &colony_props, transform.translation.truncate(), ants_to_spawn);
                    total_spawned += spawned_ants.len();
                    
                    debug!(
                        "Spawned {} ants for colony at ({:.2}, {:.2}), new population: {}",
                        spawned_ants.len(), transform.translation.x, transform.translation.y, colony_props.population + spawned_ants.len() as i32
                    );
                    
                    // Consume resources for all spawned ants
                    let old_resources = resources.resources.clone();
                    for _ in 0..spawned_ants.len() {
                        consume_spawning_resources(&mut resources.resources, &nest.upgrade_cost);
                    }
                    
                    debug!(
                        "Consumed spawning resources for {} ants: {:?} -> {:?}",
                        spawned_ants.len(), old_resources, resources.resources
                    );
                }
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
    debug!("colony_spawning_system returning");
}

/// System to manage colony resource consumption
#[instrument(skip(colonies, simulation_state))]
pub fn colony_resource_consumption_system(
    mut colonies: Query<(&mut ColonyResources, &ColonyProperties), With<Colony>>,
    simulation_state: Res<SimulationState>,
) {
    debug!("running colony_resource_consumption_system");
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
    debug!("colony_resource_consumption_system returning");
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
    debug!("running colony_upgrade_system");
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
    debug!("colony_upgrade_system returning");
}

/*
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
*/

/// System to manage colony statistics
#[instrument(skip(stats, colonies, ants))]
pub fn colony_stats_system(
    mut stats: ResMut<SimulationStats>,
    colonies: Query<&ColonyProperties, With<Colony>>,
    ants: Query<&Ant, With<Ant>>,
) {
    debug!("running colony_stats_system");
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
    debug!("colony_stats_system returning");
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
    debug!("running has_sufficient_resources");
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
    debug!("has_sufficient_resources returning true");
    true
}

/// Consume resources for spawning/upgrading
#[instrument(skip(resources, cost))]
fn consume_spawning_resources(
    resources: &mut HashMap<String, f32>,
    cost: &HashMap<String, f32>,
) {
    debug!("running consume_spawning_resources");
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
    debug!("consume_spawning_resources returning");
}

/// Spawn multiple ants for a colony in a batch with Big Brain AI
#[instrument(skip(commands, colony_props))]
fn spawn_ant_batch_with_ai(
    commands: &mut Commands,
    colony_props: &ColonyProperties,
    colony_position: Vec2,
    count: i32,
) -> Vec<Entity> {
    debug!("running spawn_ant_batch_with_ai for {} ants", count);
    let mut spawned_ants = Vec::with_capacity(count as usize);
    let mut rng = rand::thread_rng();
    
    for i in 0..count {
        // Generate random position near colony center
        let angle = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
        let distance = rng.gen::<f32>() * colony_props.radius;
        let spawn_position = colony_position + Vec2::new(angle.cos(), angle.sin()) * distance;

        debug!(
            "Batch spawning ant {}/{} with AI: colony_pos=({:.2}, {:.2}), spawn_pos=({:.2}, {:.2}), angle={:.2}, distance={:.2}",
            i + 1, count, colony_position.x, colony_position.y, spawn_position.x, spawn_position.y, angle, distance
        );

        // Use the Big Brain spawning function
        let ant_entity = crate::managers::ant_spawn::spawn_ant_with_big_brain(commands, spawn_position, None);
        
        // Update the ant to have the colony's color (will be applied next frame)
        // We'll use a system to update the color since Commands defer the changes

        spawned_ants.push(ant_entity);
    }

    info!(
        "🐜 Batch spawned {} ants with Big Brain AI, color_hue: {:.2}",
        spawned_ants.len(), colony_props.color_hue
    );

    debug!("spawn_ant_batch_with_ai returning");
    spawned_ants
}

/// Spawn a new ant for a colony
#[instrument(skip(commands, colony_props))]
fn spawn_ant(
    commands: &mut Commands,
    colony_props: &ColonyProperties,
    colony_position: Vec2,
) -> Entity {
    debug!("running spawn_ant");
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
            desired_direction: Vec2::new(1.0, 0.0),
            momentum: 0.95,
            last_positions: Vec::new(),
            turn_smoothness: 3.0,
            wander_angle: 0.0,
            wander_change: 0.3,
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
            visited_positions: Vec::new(),
            last_stuck_check: 0,
            stuck_counter: 0,
            exploration_radius: 100.0,
            path_history: Vec::new(),
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

    debug!("spawn_ant returning");
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
            //colony_territory_system,
            colony_stats_system,
        ));
        debug!("ColonyPlugin systems registered");
    }
} 