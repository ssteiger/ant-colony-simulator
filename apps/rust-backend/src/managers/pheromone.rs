use bevy::prelude::*;
use crate::models::*;
use tracing::{debug, info, warn, instrument};

// ============================================================================
// PHEROMONE MANAGEMENT SYSTEMS
// ============================================================================

/// System to manage pheromone trail decay
#[instrument(skip(commands, pheromones, simulation_state))]
pub fn pheromone_decay_system(
    mut commands: Commands,
    mut pheromones: Query<(Entity, &mut PheromoneProperties), With<PheromoneTrail>>,
    simulation_state: Res<SimulationState>,
) {
    let mut decayed_count = 0;
    let mut removed_count = 0;

    for (entity, mut pheromone) in pheromones.iter_mut() {
        let old_strength = pheromone.strength;
        
        // Decay strength over time
        pheromone.strength = (pheromone.strength - pheromone.decay_rate).max(0.0);
        
        if pheromone.strength != old_strength {
            decayed_count += 1;
            debug!(
                "Pheromone {:?} decayed: {:.2} -> {:.2} (type: {:?})",
                entity, old_strength, pheromone.strength, pheromone.trail_type
            );
        }
        
        // Remove if expired or too weak
        if pheromone.strength <= 0.0 || simulation_state.current_tick > pheromone.expires_at {
            commands.entity(entity).despawn();
            removed_count += 1;
            debug!(
                "Removed pheromone {:?} (strength: {:.2}, expires_at: {}, current_tick: {})",
                entity, pheromone.strength, pheromone.expires_at, simulation_state.current_tick
            );
        }
    }

    if decayed_count > 0 || removed_count > 0 {
        debug!(
            "Pheromone decay: {} decayed, {} removed, tick: {}",
            decayed_count, removed_count, simulation_state.current_tick
        );
    }
}

/// System to create pheromone trails from ant movement
#[instrument(skip(commands, ants, simulation_state))]
pub fn pheromone_creation_system(
    mut commands: Commands,
    ants: Query<(
        &AntPhysics,
        &AntState,
        &AntMemory,
        &CarriedResources,
        Entity,
    ), With<Ant>>,
    simulation_state: Res<SimulationState>,
) {
    let mut created_count = 0;

    for (physics, state, memory, resources, ant_entity) in ants.iter() {
        // Create pheromones continuously when ant is moving (precise movement tracking)
        if physics.velocity.length() > 0.05 && // Lower threshold for more sensitive detection
           should_create_pheromone(state, resources, simulation_state.current_tick, memory) {
            
            let (pheromone_type, strength, lifespan) = calculate_pheromone_properties(state, resources);

            // Create pheromone trail at current position for precise tracking
            let pheromone_entity = commands.spawn((
                PheromoneTrail,
                PheromoneProperties {
                    trail_type: pheromone_type.clone(),
                    strength,
                    max_strength: strength,
                    decay_rate: calculate_decay_rate(&pheromone_type),
                    expires_at: simulation_state.current_tick + lifespan,
                    source_ant: Some(ant_entity),
                    target_food: None,
                },
                Transform::from_translation(Vec3::new(physics.position.x, physics.position.y, 0.0)),
            )).id();

            created_count += 1;
            
            // Also create pheromones along recent path for continuous trails
            // Use last few positions to fill gaps between current and previous locations
            if physics.last_positions.len() > 0 {
                // Get the most recent position from history
                let last_pos = physics.last_positions[physics.last_positions.len() - 1];
                let current_pos = physics.position;
                
                // Calculate distance moved
                let distance = (current_pos - last_pos).length();
                
                // If ant moved more than 3 pixels, create intermediate pheromones for continuity
                if distance > 3.0 {
                    let steps = (distance / 2.0).ceil() as i32; // Create pheromone every ~2 pixels
                    for i in 1..steps {
                        let t = i as f32 / steps as f32;
                        let interpolated_pos = last_pos.lerp(current_pos, t);
                        
                        commands.spawn((
                            PheromoneTrail,
                            PheromoneProperties {
                                trail_type: pheromone_type.clone(),
                                strength: strength * 0.8, // Slightly weaker for interpolated positions
                                max_strength: strength,
                                decay_rate: calculate_decay_rate(&pheromone_type),
                                expires_at: simulation_state.current_tick + lifespan,
                                source_ant: Some(ant_entity),
                                target_food: None,
                            },
                            Transform::from_translation(Vec3::new(interpolated_pos.x, interpolated_pos.y, 0.0)),
                        ));
                        created_count += 1;
                    }
                }
            }

            debug!(
                "Created continuous pheromone trail {:?} at ({:.2}, {:.2}) type: {:?}, strength: {:.2}, lifespan: {}, total: {}",
                pheromone_entity, physics.position.x, physics.position.y, pheromone_type, strength, lifespan, created_count
            );
        }
    }

    if created_count > 0 {
        debug!("Created {} pheromone trails, tick: {}", created_count, simulation_state.current_tick);
    }
}

/// Determine if an ant should create a pheromone at this moment
fn should_create_pheromone(
    state: &AntState,
    resources: &CarriedResources,
    current_tick: i64,
    memory: &AntMemory,
) -> bool {
    match state {
        AntState::CarryingFood => {
            // Create strong pheromones when carrying food back to colony
            !resources.resources.is_empty()
        }
        AntState::SeekingFood => {
            // Create weaker exploration pheromones when seeking food
            current_tick % 10 == 0 // Less frequent for exploration
        }
        AntState::Exploring => {
            // Create exploration pheromones occasionally
            current_tick % 15 == 0
        }
        AntState::Following => {
            // Don't create pheromones when following (to avoid loops)
            false
        }
        _ => false,
    }
}

/// Calculate pheromone properties based on ant state
fn calculate_pheromone_properties(
    state: &AntState,
    resources: &CarriedResources,
) -> (PheromoneType, f32, i64) {
    match state {
        AntState::CarryingFood => {
            // Strong food pheromones with 10x longer lifespan for precise tracking
            let strength = 90.0 + (resources.current_weight * 2.0).min(50.0);
            (PheromoneType::Food, strength, 20000) // 10x lifespan: 2000 -> 20000
        }
        AntState::SeekingFood => {
            // Weaker exploration pheromones with 10x longer visibility
            (PheromoneType::Exploration, 40.0, 12000) // 10x lifespan: 1200 -> 12000
        }
        AntState::Exploring => {
            // Basic exploration pheromones with 10x longer visibility
            (PheromoneType::Exploration, 35.0, 10000) // 10x lifespan: 1000 -> 10000
        }
        AntState::Following => {
            // Home pheromones for ants following trails back to colony with 10x lifespan
            (PheromoneType::Home, 50.0, 15000) // 10x lifespan: 1500 -> 15000
        }
        _ => {
            // Default weak pheromone with 10x longer visibility
            (PheromoneType::Exploration, 30.0, 8000) // 10x lifespan: 800 -> 8000
        }
    }
}

/// Calculate decay rate based on pheromone type - adjusted for 10x longer lifespans
fn calculate_decay_rate(pheromone_type: &PheromoneType) -> f32 {
    match pheromone_type {
        PheromoneType::Food => 0.04,     // 10x slower decay for persistent food trails
        PheromoneType::Home => 0.03,     // 10x slower decay for very persistent home trails  
        PheromoneType::Exploration => 0.06, // 10x slower decay for exploration trails
        PheromoneType::Danger => 0.15,  // 10x slower decay but still relatively fast
    }
}

/// System to handle ant pheromone detection and following
#[instrument(skip(ants, pheromones))]
pub fn pheromone_detection_system(
    mut ants: Query<(
        &mut AntPhysics,
        &mut AntTarget,
        &mut AntMemory,
        &AntState,
        Entity,
    ), With<Ant>>,
    pheromones: Query<(&PheromoneProperties, &Transform), With<PheromoneTrail>>,
) {
    let mut ants_following = 0;
    let pheromone_count = pheromones.iter().count();

    for (mut physics, mut target, mut memory, state, ant_entity) in ants.iter_mut() {
        // Only follow pheromones if seeking food or exploring
        if !matches!(state, AntState::SeekingFood | AntState::Exploring | AntState::Wandering) {
            continue;
        }

        let pheromone_influence = calculate_pheromone_influence(
            &physics,
            &memory,
            state,
            &pheromones
        );

        // Apply pheromone influence to movement
        if let Some(influence) = pheromone_influence {
            // Set the desired direction based on pheromone gradient
            physics.desired_direction = influence.direction;
            
            // Adjust target based on pheromone strength and type
            if influence.strength > 30.0 {
                // Strong pheromone trail - follow it closely
                let follow_target = physics.position + influence.direction * 40.0;
                *target = AntTarget::Position(follow_target);
                ants_following += 1;
                
                debug!(
                    "Ant {:?} following strong pheromone trail (strength: {:.2}) towards ({:.2}, {:.2})",
                    ant_entity, influence.strength, follow_target.x, follow_target.y
                );
            } else if influence.strength > 10.0 {
                // Moderate pheromone - influence movement direction but don't override specific targets
                if matches!(*target, AntTarget::None) {
                    let gentle_target = physics.position + influence.direction * 20.0;
                    *target = AntTarget::Position(gentle_target);
                    ants_following += 1;
                    
                    debug!(
                        "Ant {:?} gently influenced by pheromone (strength: {:.2})",
                        ant_entity, influence.strength
                    );
                }
            }
            
            // Update memory about pheromone following
            memory.last_action_tick = memory.last_action_tick.max(
                memory.last_action_tick.saturating_add(1)
            );
        }
    }

    if ants_following > 0 {
        debug!(
            "Pheromone detection: {} ants following pheromones out of {} total pheromones",
            ants_following, pheromone_count
        );
    }
}

/// Calculate the combined influence of nearby pheromones
fn calculate_pheromone_influence(
    physics: &AntPhysics,
    memory: &AntMemory,
    state: &AntState,
    pheromones: &Query<(&PheromoneProperties, &Transform), With<PheromoneTrail>>,
) -> Option<PheromoneInfluence> {
    let mut total_direction = Vec2::ZERO;
    let mut total_strength = 0.0;
    let mut pheromone_count = 0;

    // Detection range based on ant's pheromone sensitivity
    let base_detection_range = 60.0 * memory.pheromone_sensitivity;

    for (pheromone, pheromone_transform) in pheromones.iter() {
        let pheromone_pos = pheromone_transform.translation.truncate();
        let distance = physics.position.distance(pheromone_pos);
        
        // Adjust detection range based on pheromone type and ant state
        let detection_range = match (state, &pheromone.trail_type) {
            (AntState::SeekingFood, PheromoneType::Food) => base_detection_range * 1.5,
            (AntState::CarryingFood, PheromoneType::Home) => base_detection_range * 1.3,
            (AntState::Exploring, PheromoneType::Exploration) => base_detection_range * 1.2,
            _ => base_detection_range,
        };
        
        if distance < detection_range && distance > 0.1 {
            // Calculate influence based on distance and strength
            let distance_factor = 1.0 - (distance / detection_range);
            let influence_strength = pheromone.strength * distance_factor;
            
            // Create gradient towards pheromone
            let direction_to_pheromone = (pheromone_pos - physics.position).normalize_or_zero();
            
            // Weight by pheromone type preference
            let type_weight = match (state, &pheromone.trail_type) {
                (AntState::SeekingFood, PheromoneType::Food) => 2.0,
                (AntState::CarryingFood, PheromoneType::Home) => 2.0,
                (AntState::Exploring, PheromoneType::Exploration) => 1.5,
                (_, PheromoneType::Danger) => -1.0, // Avoid danger pheromones
                _ => 1.0,
            };
            
            let weighted_strength = influence_strength * type_weight;
            
            if weighted_strength > 0.0 {
                total_direction += direction_to_pheromone * weighted_strength;
                total_strength += weighted_strength;
                pheromone_count += 1;
                
                debug!(
                    "Pheromone at ({:.2}, {:.2}) influencing ant: type={:?}, strength={:.2}, distance={:.2}, weight={:.2}",
                    pheromone_pos.x, pheromone_pos.y, pheromone.trail_type, influence_strength, distance, type_weight
                );
            }
        }
    }

    if pheromone_count > 0 && total_strength > 5.0 {
        // Average the direction weighted by strength
        let average_direction = (total_direction / total_strength).normalize_or_zero();
        
        Some(PheromoneInfluence {
            direction: average_direction,
            strength: total_strength,
            pheromone_count,
        })
    } else {
        None
    }
}

/// Represents the combined influence of pheromones on an ant
struct PheromoneInfluence {
    direction: Vec2,
    strength: f32,
    pheromone_count: i32,
}

/// System to merge nearby pheromone trails
#[instrument(skip(commands, pheromones))]
pub fn pheromone_merging_system(
    mut commands: Commands,
    mut pheromones: Query<(Entity, &mut PheromoneProperties, &Transform), With<PheromoneTrail>>,
) {
    let pheromone_positions: Vec<(Entity, PheromoneProperties, Vec2)> = pheromones
        .iter()
        .map(|(entity, props, transform)| {
            (
                entity,
                props.clone(),
                transform.translation.truncate(),
            )
        })
        .collect();

    let mut merged_entities = Vec::new();
    let mut merge_count = 0;

    for (i, (entity1, props1, pos1)) in pheromone_positions.iter().enumerate() {
        for (entity2, props2, pos2) in pheromone_positions.iter().skip(i + 1) {
            let distance = pos1.distance(*pos2);
            
            // Merge pheromones if they're close and of the same type
            if distance < 5.0 && props1.trail_type == props2.trail_type {
                merge_count += 1;
                debug!(
                    "Merging pheromones: {:?} ({:.2}, {:.2}) and {:?} ({:.2}, {:.2}) at distance {:.2}",
                    entity1, pos1.x, pos1.y, entity2, pos2.x, pos2.y, distance
                );
                
                // Merge into the stronger pheromone
                if props1.strength >= props2.strength {
                    merged_entities.push(*entity2);
                    debug!("Keeping {:?} (strength: {:.2}), removing {:?} (strength: {:.2})", 
                           entity1, props1.strength, entity2, props2.strength);
                } else {
                    merged_entities.push(*entity1);
                    debug!("Keeping {:?} (strength: {:.2}), removing {:?} (strength: {:.2})", 
                           entity2, props2.strength, entity1, props1.strength);
                }
            }
        }
    }

    // Remove merged entities
    let removed_count = merged_entities.len();
    for entity in merged_entities {
        commands.entity(entity).despawn();
    }

    if merge_count > 0 {
        debug!("Merged {} pheromone pairs, removed {} entities", merge_count, removed_count);
    }
}

/// System to create danger pheromones
#[instrument(skip(commands, ants, simulation_state))]
pub fn danger_pheromone_system(
    mut commands: Commands,
    ants: Query<(&AntPhysics, &AntHealth), With<Ant>>,
    simulation_state: Res<SimulationState>,
) {
    let mut danger_pheromones_created = 0;

    for (physics, health) in ants.iter() {
        // Create danger pheromone if ant is critically injured
        if health.health < 20.0 {
            let danger_entity = commands.spawn((
                PheromoneTrail,
                PheromoneProperties {
                    trail_type: PheromoneType::Danger,
                    strength: 150.0,
                    max_strength: 150.0,
                    decay_rate: 2.0,
                    expires_at: simulation_state.current_tick + 2000,
                    source_ant: None,
                    target_food: None,
                },
                Transform::from_translation(Vec3::new(physics.position.x, physics.position.y, 0.0)),
            )).id();

            danger_pheromones_created += 1;
            warn!(
                "Created danger pheromone {:?} at ({:.2}, {:.2}) - ant health: {:.2}",
                danger_entity, physics.position.x, physics.position.y, health.health
            );
        }
    }

    if danger_pheromones_created > 0 {
        warn!(
            "Created {} danger pheromones due to injured ants, tick: {}",
            danger_pheromones_created, simulation_state.current_tick
        );
    }
}

/// System to handle ant avoidance of danger pheromones
#[instrument(skip(ants, danger_pheromones))]
pub fn danger_avoidance_system(
    mut ants: Query<(&mut AntPhysics, &mut AntTarget, &AntMemory), With<Ant>>,
    danger_pheromones: Query<(&PheromoneProperties, &Transform), (With<PheromoneTrail>, With<PheromoneType>)>,
) {
    let mut ants_avoiding = 0;

    for (mut physics, mut target, memory) in ants.iter_mut() {
        let mut danger_direction = Vec2::ZERO;
        let mut danger_count = 0;

        // Find nearby danger pheromones
        for (pheromone, pheromone_transform) in danger_pheromones.iter() {
            if pheromone.trail_type == PheromoneType::Danger {
                let pheromone_pos = pheromone_transform.translation.truncate();
                let distance = physics.position.distance(pheromone_pos);
                
                let detection_range = 30.0 * memory.pheromone_sensitivity;
                
                if distance < detection_range {
                    danger_count += 1;
                    // Calculate avoidance direction
                    let direction = (physics.position - pheromone_pos).normalize();
                    let strength = pheromone.strength * (1.0 - distance / detection_range);
                    
                    danger_direction += direction * strength;
                    
                    debug!(
                        "Ant at ({:.2}, {:.2}) avoiding danger pheromone at ({:.2}, {:.2}), distance: {:.2}, strength: {:.2}",
                        physics.position.x, physics.position.y, pheromone_pos.x, pheromone_pos.y, distance, strength
                    );
                }
            }
        }

        // Apply avoidance behavior
        if danger_count > 0 {
            danger_direction = danger_direction / danger_count as f32;
            
            // Set target to move away from danger
            let avoidance_target = physics.position + danger_direction * 50.0;
            *target = AntTarget::Position(avoidance_target);
            ants_avoiding += 1;
            
            debug!(
                "Ant avoiding {} danger pheromones, moving from ({:.2}, {:.2}) to ({:.2}, {:.2})",
                danger_count, physics.position.x, physics.position.y, avoidance_target.x, avoidance_target.y
            );
        }
    }

    if ants_avoiding > 0 {
        debug!("{} ants avoiding danger pheromones", ants_avoiding);
    }
}

/// System to update pheromone statistics
#[instrument(skip(stats, pheromones))]
pub fn pheromone_stats_system(
    mut stats: ResMut<SimulationStats>,
    pheromones: Query<&PheromoneProperties, With<PheromoneTrail>>,
) {
    let old_count = stats.pheromone_trail_count;
    stats.pheromone_trail_count = pheromones.iter().count() as i32;
    
    if stats.pheromone_trail_count != old_count {
        debug!(
            "Pheromone trail count updated: {} -> {}",
            old_count, stats.pheromone_trail_count
        );
    }
}

/// System to visualize pheromone trails (for debugging)
#[instrument(skip(pheromones))]
pub fn pheromone_visualization_system(
    pheromones: Query<(&PheromoneProperties, &Transform), With<PheromoneTrail>>,
) {
    let mut type_counts = std::collections::HashMap::new();
    
    for (pheromone, transform) in pheromones.iter() {
        // Count pheromones by type for debugging
        *type_counts.entry(pheromone.trail_type).or_insert(0) += 1;
        
        // This would be used for rendering pheromone trails
        // In a real implementation, you'd update sprite colors or create visual effects
        let alpha = pheromone.strength / pheromone.max_strength;
        let color = match pheromone.trail_type {
            PheromoneType::Food => Color::YELLOW,
            PheromoneType::Danger => Color::RED,
            PheromoneType::Home => Color::BLUE,
            PheromoneType::Exploration => Color::YELLOW,
        };
        
        // Apply alpha to color
        let _visual_color = color.with_a(alpha);
    }
    
    // Log pheromone type distribution periodically
    if !type_counts.is_empty() {
        debug!("Pheromone type distribution: {:?}", type_counts);
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Calculate pheromone strength based on distance and base strength
pub fn calculate_pheromone_strength(base_strength: f32, distance: f32, max_distance: f32) -> f32 {
    let distance_factor = 1.0 - (distance / max_distance).clamp(0.0, 1.0);
    let strength = base_strength * distance_factor;
    
    debug!(
        "Calculated pheromone strength: base={:.2}, distance={:.2}, max_distance={:.2}, result={:.2}",
        base_strength, distance, max_distance, strength
    );
    
    strength
}

/// Check if two pheromone trails should be merged
pub fn should_merge_pheromones(
    pos1: Vec2,
    pos2: Vec2,
    type1: PheromoneType,
    type2: PheromoneType,
    merge_distance: f32,
) -> bool {
    let distance = pos1.distance(pos2);
    let should_merge = distance < merge_distance && type1 == type2;
    
    debug!(
        "Merge check: distance={:.2}, merge_distance={:.2}, types: {:?} vs {:?}, should_merge={}",
        distance, merge_distance, type1, type2, should_merge
    );
    
    should_merge
}

/// Plugin for pheromone management systems
pub struct PheromonePlugin;

impl Plugin for PheromonePlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing PheromonePlugin");
        app.add_systems(Update, (
            pheromone_decay_system,
            pheromone_creation_system,
            pheromone_detection_system,
            pheromone_merging_system,
            // danger_pheromone_system,  // Temporarily deactivated
            danger_avoidance_system,
            pheromone_stats_system,
            pheromone_visualization_system,
        ));
        debug!("PheromonePlugin systems registered");
    }
} 