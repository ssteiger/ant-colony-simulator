use bevy::prelude::*;
use crate::models::*;

// ============================================================================
// PHEROMONE MANAGEMENT SYSTEMS
// ============================================================================

/// System to manage pheromone trail decay
pub fn pheromone_decay_system(
    mut commands: Commands,
    mut pheromones: Query<(Entity, &mut PheromoneProperties), With<PheromoneTrail>>,
    simulation_state: Res<SimulationState>,
) {
    for (entity, mut pheromone) in pheromones.iter_mut() {
        // Decay strength over time
        pheromone.strength = (pheromone.strength - pheromone.decay_rate).max(0.0);
        
        // Remove if expired or too weak
        if pheromone.strength <= 0.0 || simulation_state.current_tick > pheromone.expires_at {
            commands.entity(entity).despawn();
        }
    }
}

/// System to create pheromone trails from ant movement
pub fn pheromone_creation_system(
    mut commands: Commands,
    ants: Query<(
        &AntPhysics,
        &AntState,
        &AntMemory,
    ), With<Ant>>,
    simulation_state: Res<SimulationState>,
) {
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
            commands.spawn((
                PheromoneTrail,
                PheromoneProperties {
                    trail_type: pheromone_type,
                    strength: 100.0,
                    max_strength: 100.0,
                    decay_rate: 1.0,
                    expires_at: simulation_state.current_tick + 1000,
                    source_ant: None,
                    target_food: None,
                },
                Transform::from_translation(Vec3::new(physics.position.x, physics.position.y, 0.0)),
            ));
        }
    }
}

/// System to handle ant pheromone detection and following
pub fn pheromone_detection_system(
    mut ants: Query<(
        &mut AntPhysics,
        &mut AntTarget,
        &AntMemory,
        &AntState,
    ), With<Ant>>,
    pheromones: Query<(&PheromoneProperties, &Transform), With<PheromoneTrail>>,
) {
    for (mut physics, mut target, memory, state) in ants.iter_mut() {
        // Only follow pheromones if seeking food or exploring
        if !matches!(state, AntState::SeekingFood | AntState::Exploring) {
            continue;
        }

        let mut best_pheromone = None;
        let mut best_score = 0.0;

        // Find the strongest pheromone within detection range
        for (pheromone, pheromone_transform) in pheromones.iter() {
            let pheromone_pos = pheromone_transform.translation.truncate();
            let distance = physics.position.distance(pheromone_pos);
            
            // Detection range based on ant's pheromone sensitivity
            let detection_range = 50.0 * memory.pheromone_sensitivity;
            
            if distance < detection_range {
                // Calculate pheromone score based on strength and distance
                let distance_factor = 1.0 - (distance / detection_range);
                let score = pheromone.strength * distance_factor;
                
                if score > best_score {
                    best_score = score;
                    best_pheromone = Some(pheromone_pos);
                }
            }
        }

        // Follow the best pheromone if found
        if let Some(pheromone_pos) = best_pheromone {
            *target = AntTarget::Position(pheromone_pos);
        }
    }
}

/// System to merge nearby pheromone trails
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

    for (i, (entity1, props1, pos1)) in pheromone_positions.iter().enumerate() {
        for (entity2, props2, pos2) in pheromone_positions.iter().skip(i + 1) {
            let distance = pos1.distance(*pos2);
            
            // Merge pheromones if they're close and of the same type
            if distance < 5.0 && props1.trail_type == props2.trail_type {
                // Merge into the stronger pheromone
                if props1.strength >= props2.strength {
                    merged_entities.push(*entity2);
                } else {
                    merged_entities.push(*entity1);
                }
            }
        }
    }

    // Remove merged entities
    for entity in merged_entities {
        commands.entity(entity).despawn();
    }
}

/// System to create danger pheromones
pub fn danger_pheromone_system(
    mut commands: Commands,
    ants: Query<(&AntPhysics, &AntHealth), With<Ant>>,
    simulation_state: Res<SimulationState>,
) {
    for (physics, health) in ants.iter() {
        // Create danger pheromone if ant is critically injured
        if health.health < 20.0 {
            commands.spawn((
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
            ));
        }
    }
}

/// System to handle ant avoidance of danger pheromones
pub fn danger_avoidance_system(
    mut ants: Query<(&mut AntPhysics, &mut AntTarget, &AntMemory), With<Ant>>,
    danger_pheromones: Query<(&PheromoneProperties, &Transform), (With<PheromoneTrail>, With<PheromoneType>)>,
) {
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
                    // Calculate avoidance direction
                    let direction = (physics.position - pheromone_pos).normalize();
                    let strength = pheromone.strength * (1.0 - distance / detection_range);
                    
                    danger_direction += direction * strength;
                    danger_count += 1;
                }
            }
        }

        // Apply avoidance behavior
        if danger_count > 0 {
            danger_direction = danger_direction / danger_count as f32;
            
            // Set target to move away from danger
            let avoidance_target = physics.position + danger_direction * 50.0;
            *target = AntTarget::Position(avoidance_target);
        }
    }
}

/// System to update pheromone statistics
pub fn pheromone_stats_system(
    mut stats: ResMut<SimulationStats>,
    pheromones: Query<&PheromoneProperties, With<PheromoneTrail>>,
) {
    stats.pheromone_trail_count = pheromones.iter().count() as i32;
}

/// System to visualize pheromone trails (for debugging)
pub fn pheromone_visualization_system(
    pheromones: Query<(&PheromoneProperties, &Transform), With<PheromoneTrail>>,
) {
    for (pheromone, transform) in pheromones.iter() {
        // This would be used for rendering pheromone trails
        // In a real implementation, you'd update sprite colors or create visual effects
        let alpha = pheromone.strength / pheromone.max_strength;
        let color = match pheromone.trail_type {
            PheromoneType::Food => Color::GREEN,
            PheromoneType::Danger => Color::RED,
            PheromoneType::Home => Color::BLUE,
            PheromoneType::Exploration => Color::YELLOW,
        };
        
        // Apply alpha to color
        let _visual_color = color.with_a(alpha);
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Calculate pheromone strength based on distance and base strength
pub fn calculate_pheromone_strength(base_strength: f32, distance: f32, max_distance: f32) -> f32 {
    let distance_factor = 1.0 - (distance / max_distance).clamp(0.0, 1.0);
    base_strength * distance_factor
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
    distance < merge_distance && type1 == type2
}

/// Plugin for pheromone management systems
pub struct PheromonePlugin;

impl Plugin for PheromonePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            pheromone_decay_system,
            pheromone_creation_system,
            pheromone_detection_system,
            pheromone_merging_system,
            danger_pheromone_system,
            danger_avoidance_system,
            pheromone_stats_system,
            pheromone_visualization_system,
        ));
    }
} 