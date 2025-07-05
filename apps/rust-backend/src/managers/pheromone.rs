use bevy::prelude::*;
use crate::models::*;
use tracing::{debug, info, warn, error, instrument};

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
    ), With<Ant>>,
    simulation_state: Res<SimulationState>,
) {
    let mut created_count = 0;

    for (physics, state, memory) in ants.iter() {
        // Only create pheromones if ant is moving
        if physics.velocity.length() > 0.1 {
            let pheromone_type = match state {
                AntState::CarryingFood => PheromoneType::Food,
                AntState::SeekingFood => PheromoneType::Food,
                AntState::Exploring => PheromoneType::Exploration,
                AntState::Patrolling => PheromoneType::Home,
                _ => continue, // Don't create pheromones for other states
            };

            // Create pheromone trail
            let pheromone_entity = commands.spawn((
                PheromoneTrail,
                PheromoneProperties {
                    trail_type: pheromone_type.clone(),
                    strength: 100.0,
                    max_strength: 100.0,
                    decay_rate: 1.0,
                    expires_at: simulation_state.current_tick + 1000,
                    source_ant: None,
                    target_food: None,
                },
                Transform::from_translation(Vec3::new(physics.position.x, physics.position.y, 0.0)),
            )).id();

            created_count += 1;
            debug!(
                "Created pheromone {:?} at ({:.2}, {:.2}) type: {:?}, state: {:?}",
                pheromone_entity, physics.position.x, physics.position.y, pheromone_type, state
            );
        }
    }

    if created_count > 0 {
        debug!("Created {} pheromone trails, tick: {}", created_count, simulation_state.current_tick);
    }
}

/// System to handle ant pheromone detection and following
#[instrument(skip(ants, pheromones))]
pub fn pheromone_detection_system(
    mut ants: Query<(
        &mut AntPhysics,
        &mut AntTarget,
        &AntMemory,
        &AntState,
    ), With<Ant>>,
    pheromones: Query<(&PheromoneProperties, &Transform), With<PheromoneTrail>>,
) {
    let mut ants_following = 0;
    let pheromone_count = pheromones.iter().count();

    for (mut physics, mut target, memory, state) in ants.iter_mut() {
        // Only follow pheromones if seeking food or exploring
        if !matches!(state, AntState::SeekingFood | AntState::Exploring) {
            continue;
        }

        let mut best_pheromone = None;
        let mut best_score = 0.0;
        let mut detected_count = 0;

        // Find the strongest pheromone within detection range
        for (pheromone, pheromone_transform) in pheromones.iter() {
            let pheromone_pos = pheromone_transform.translation.truncate();
            let distance = physics.position.distance(pheromone_pos);
            
            // Detection range based on ant's pheromone sensitivity
            let detection_range = 50.0 * memory.pheromone_sensitivity;
            
            if distance < detection_range {
                detected_count += 1;
                // Calculate pheromone score based on strength and distance
                let distance_factor = 1.0 - (distance / detection_range);
                let score = pheromone.strength * distance_factor;
                
                debug!(
                    "Ant at ({:.2}, {:.2}) detected pheromone {:?} at distance {:.2}, score: {:.2}",
                    physics.position.x, physics.position.y, pheromone.trail_type, distance, score
                );
                
                if score > best_score {
                    best_score = score;
                    best_pheromone = Some(pheromone_pos);
                }
            }
        }

        // Follow the best pheromone if found
        if let Some(pheromone_pos) = best_pheromone {
            *target = AntTarget::Position(pheromone_pos);
            ants_following += 1;
            debug!(
                "Ant at ({:.2}, {:.2}) following pheromone to ({:.2}, {:.2}), detected {} pheromones",
                physics.position.x, physics.position.y, pheromone_pos.x, pheromone_pos.y, detected_count
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