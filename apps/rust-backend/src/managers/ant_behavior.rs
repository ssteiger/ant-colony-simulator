use crate::models::*;
use bevy::prelude::*;
use big_brain::prelude::*;
use tracing::{debug, info, trace, warn};

// ============================================================================
// BIG-BRAIN SCORERS - Decision Inputs for Ant AI
// ============================================================================

/// Scorer that evaluates if ant is a scout (explores for food)
#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct ScoutScorer;

/// Scorer that evaluates if an ant is carrying food
#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct CarryingFoodScorer;

/// Scorer that evaluates proximity to food sources
#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct NearFoodScorer;

/// Scorer that evaluates proximity to colony
#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct NearColonyScorer;

/// Scorer that evaluates pheromone trail strength
#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct PheromoneStrengthScorer;

/// Scorer that evaluates exploration urge (based on recent positions)
#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct ExplorationUrgeScorer;

/// Scorer that evaluates if ant needs rest (very low energy)
#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct NeedsRestScorer;

/// Scorer that evaluates if ant is stuck and needs escape behavior
#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct StuckScorer;

// ============================================================================
// BIG-BRAIN ACTIONS - Behaviors for Ants
// ============================================================================

/// Action for wandering around randomly
#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct WanderAction;

/// Action for seeking food sources
#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct SeekFoodAction;

/// Action for collecting food when near a food source
#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct CollectFoodAction;

/// Action for returning to colony with food
#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct ReturnToColonyAction;

/// Action for following pheromone trails
#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct FollowPheromoneAction;

/// Action for exploring new areas
#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct ExploreAction;

/// Action for depositing food at colony
#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct DepositFoodAction;

/// Action for resting to recover energy
#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct RestAction;

/// Action for escaping when stuck
#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct EscapeAction;

/// Action for scout ants to mark found food sources
#[derive(Component, Debug, Clone, ActionBuilder)]
pub struct MarkFoodSourceAction;

// ============================================================================
// SCORER SYSTEMS - Evaluate World State
// ============================================================================

/// System that scores if ant is a scout (role-based behavior)
pub fn scout_scorer_system(
    ant_types: Query<&AntType>,
    mut scorers: Query<(&Actor, &mut Score), With<ScoutScorer>>,
) {
    for (Actor(actor), mut score) in scorers.iter_mut() {
        if let Ok(ant_type) = ant_types.get(*actor) {
            // Scout ants get high score for exploration behavior
            let scout_score = if ant_type.role == "scout" { 1.0 } else { 0.0 };
            score.set(scout_score);
            
            if scout_score > 0.0 {
                trace!("Ant {:?} is a scout (score: {:.2})", actor, scout_score);
            }
        }
    }
}

/// System that scores if an ant is carrying food
pub fn carrying_food_scorer_system(
    carried_resources: Query<&CarriedResources>,
    mut scorers: Query<(&Actor, &mut Score), With<CarryingFoodScorer>>,
) {
    for (Actor(actor), mut score) in scorers.iter_mut() {
        if let Ok(resources) = carried_resources.get(*actor) {
            // Simple binary score: 1.0 if carrying food, 0.0 if not
            let carrying_score = if !resources.resources.is_empty() { 1.0 } else { 0.0 };
            score.set(carrying_score);
            
            if carrying_score > 0.0 {
                trace!("Ant {:?} is carrying food (weight: {:.2})", actor, resources.current_weight);
            }
        }
    }
}

/// System that scores proximity to food sources
pub fn near_food_scorer_system(
    ant_physics: Query<&AntPhysics>,
    food_sources: Query<(&FoodSourceProperties, &Transform), With<FoodSource>>,
    mut scorers: Query<(&Actor, &mut Score), With<NearFoodScorer>>,
) {
    for (Actor(actor), mut score) in scorers.iter_mut() {
        if let Ok(physics) = ant_physics.get(*actor) {
            let mut best_score: f32 = 0.0;
            let mut closest_food_distance = f32::INFINITY;
            let food_count = food_sources.iter().count();
            
            debug!("Checking ant {:?} against {} food sources at position ({:.1}, {:.1})", 
                   actor, food_count, physics.position.x, physics.position.y);
            
            for (food_props, food_transform) in food_sources.iter() {
                if food_props.amount > 0.0 {
                    let food_pos = food_transform.translation.truncate();
                    let distance = physics.position.distance(food_pos);
                    
                    debug!("  Food at ({:.1}, {:.1}), amount: {:.1}, distance: {:.1}", 
                           food_pos.x, food_pos.y, food_props.amount, distance);
                    
                    // Score based on proximity and food amount (increased detection range)
                    let proximity_score = if distance < 200.0 {
                        let base_score = 1.0 - (distance / 200.0);
                        // Boost score based on food amount
                        base_score * (food_props.amount / 100.0).clamp(0.1, 1.0)
                    } else {
                        0.0
                    };
                    
                    if proximity_score > 0.0 {
                        debug!("    Proximity score: {:.3}", proximity_score);
                    }
                    
                    best_score = best_score.max(proximity_score);
                    closest_food_distance = closest_food_distance.min(distance);
                }
            }
            
            score.set(best_score);
            
            if best_score > 0.0 {
                info!("Ant {:?} near food (score: {:.3}, closest: {:.1})", actor, best_score, closest_food_distance);
            }
        }
    }
}

/// System that scores proximity to colony
pub fn near_colony_scorer_system(
    ant_physics: Query<&AntPhysics>,
    colonies: Query<(&ColonyProperties, &Transform), With<Colony>>,
    mut scorers: Query<(&Actor, &mut Score), With<NearColonyScorer>>,
) {
    for (Actor(actor), mut score) in scorers.iter_mut() {
        if let Ok(physics) = ant_physics.get(*actor) {
            let mut best_score: f32 = 0.0;
            
            for (colony_props, colony_transform) in colonies.iter() {
                let distance = physics.position.distance(colony_transform.translation.truncate());
                
                // Score based on proximity to colony
                let proximity_score = if distance < colony_props.radius + 50.0 {
                    1.0 - (distance / (colony_props.radius + 50.0))
                } else {
                    0.0
                };
                
                best_score = best_score.max(proximity_score);
            }
            
            score.set(best_score);
            
            if best_score > 0.5 {
                trace!("Ant {:?} near colony (score: {:.2})", actor, best_score);
            }
        }
    }
}

/// System that scores pheromone trail strength for worker ants
pub fn pheromone_strength_scorer_system(
    ant_physics: Query<&AntPhysics>,
    ant_types: Query<&AntType>,
    pheromones: Query<(&PheromoneProperties, &Transform), With<PheromoneTrail>>,
    mut scorers: Query<(&Actor, &mut Score), With<PheromoneStrengthScorer>>,
) {
    for (Actor(actor), mut score) in scorers.iter_mut() {
        if let (Ok(physics), Ok(ant_type)) = (ant_physics.get(*actor), ant_types.get(*actor)) {
            // Only worker ants follow pheromone trails
            if ant_type.role != "worker" {
                score.set(0.0);
                continue;
            }
            
            let mut strongest_pheromone: f32 = 0.0;
            
            // Find strongest nearby food pheromone trail
            for (pheromone, pheromone_transform) in pheromones.iter() {
                if pheromone.trail_type == PheromoneType::Food {
                    let pheromone_pos = pheromone_transform.translation.truncate();
                    let distance = physics.position.distance(pheromone_pos);
                    
                    // Detection range for pheromones
                    if distance < 80.0 {
                        let distance_factor = 1.0 - (distance / 80.0);
                        let pheromone_influence = pheromone.strength * distance_factor;
                        strongest_pheromone = strongest_pheromone.max(pheromone_influence);
                    }
                }
            }
            
            // Normalize to 0-1 range
            let pheromone_score = (strongest_pheromone / 100.0).clamp(0.0, 1.0);
            score.set(pheromone_score);
            
            if pheromone_score > 0.3 {
                trace!("Worker ant {:?} detects pheromone trail (score: {:.2})", actor, pheromone_score);
            }
        }
    }
}

/// System that scores exploration urge based on recent movement patterns
pub fn exploration_urge_scorer_system(
    ant_memory: Query<&AntMemory>,
    mut scorers: Query<(&Actor, &mut Score), With<ExplorationUrgeScorer>>,
) {
    for (Actor(actor), mut score) in scorers.iter_mut() {
        if let Ok(memory) = ant_memory.get(*actor) {
            // Base exploration urge
            let mut exploration_score: f32 = 0.3;
            
            // Increase urge if ant has been in similar areas recently
            if memory.visited_positions.len() > 5 {
                let recent_positions = &memory.visited_positions[memory.visited_positions.len() - 5..];
                let mut total_distance = 0.0;
                
                for i in 1..recent_positions.len() {
                    total_distance += recent_positions[i-1].distance(recent_positions[i]);
                }
                
                // If ant hasn't moved much, increase exploration urge
                if total_distance < 50.0 {
                    exploration_score += 0.4;
                }
            }
            
            // Decrease urge if ant knows about food sources
            if !memory.known_food_sources.is_empty() {
                exploration_score *= 0.7;
            }
            
            score.set(exploration_score.clamp(0.0, 1.0));
            
            if exploration_score > 0.5 {
                trace!("Ant {:?} has exploration urge (score: {:.2})", actor, exploration_score);
            }
        }
    }
}

/// System that scores if ant needs rest (based on health, not energy)
pub fn needs_rest_scorer_system(
    ant_healths: Query<&AntHealth>,
    mut scorers: Query<(&Actor, &mut Score), With<NeedsRestScorer>>,
) {
    for (Actor(actor), mut score) in scorers.iter_mut() {
        if let Ok(health) = ant_healths.get(*actor) {
            // Only need rest if health is critically low
            let rest_score = if health.health < 20.0 {
                1.0 - (health.health / 20.0)
            } else {
                0.0
            };
            
            score.set(rest_score);
            
            if rest_score > 0.0 {
                debug!("Ant {:?} needs rest (score: {:.2}, health: {:.1})", actor, rest_score, health.health);
            }
        }
    }
}

/// System that scores if ant is stuck and needs escape behavior
pub fn stuck_scorer_system(
    ant_memory: Query<&AntMemory>,
    mut scorers: Query<(&Actor, &mut Score), With<StuckScorer>>,
) {
    for (Actor(actor), mut score) in scorers.iter_mut() {
        if let Ok(memory) = ant_memory.get(*actor) {
            // Score based on stuck counter
            let stuck_score = (memory.stuck_counter as f32 / 5.0).clamp(0.0, 1.0);
            score.set(stuck_score);
            
            if stuck_score > 0.0 {
                debug!("Ant {:?} appears stuck (score: {:.2}, counter: {})", actor, stuck_score, memory.stuck_counter);
            }
        }
    }
}

// ============================================================================
// ACTION SYSTEMS - Execute Behaviors (Simplified)
// ============================================================================

/// System that handles wandering behavior
pub fn wander_action_system(
    mut ants: Query<&mut AntTarget>,
    mut action_query: Query<(&Actor, &mut ActionState), With<WanderAction>>,
) {
    for (Actor(actor), mut action_state) in action_query.iter_mut() {
        match *action_state {
            ActionState::Requested => {
                if let Ok(mut target) = ants.get_mut(*actor) {
                    *target = AntTarget::None;
                    *action_state = ActionState::Executing;
                    trace!("Ant {:?} started wandering", actor);
                }
            }
            ActionState::Executing => {
                // Wandering continues indefinitely until interrupted
            }
            ActionState::Cancelled => {
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

/// System that handles food seeking behavior
pub fn seek_food_action_system(
    mut ants: Query<(&mut AntTarget, &AntPhysics)>,
    food_sources: Query<(Entity, &FoodSourceProperties, &Transform), With<FoodSource>>,
    mut action_query: Query<(&Actor, &mut ActionState), With<SeekFoodAction>>,
) {
    for (Actor(actor), mut action_state) in action_query.iter_mut() {
        match *action_state {
            ActionState::Requested => {
                if let Ok((mut target, physics)) = ants.get_mut(*actor) {
                    // Find nearest food source with food
                    let mut nearest_food = None;
                    let mut nearest_distance = f32::INFINITY;
                    
                    for (food_entity, food_props, food_transform) in food_sources.iter() {
                        if food_props.amount > 0.0 {
                            let distance = physics.position.distance(food_transform.translation.truncate());
                            if distance < nearest_distance {
                                nearest_distance = distance;
                                nearest_food = Some(food_entity);
                            }
                        }
                    }
                    
                    if let Some(food_entity) = nearest_food {
                        *target = AntTarget::Food(food_entity);
                        *action_state = ActionState::Executing;
                        debug!("Ant {:?} seeking food source {:?}", actor, food_entity);
                    } else {
                        *action_state = ActionState::Failure;
                    }
                }
            }
            ActionState::Executing => {
                // Check if ant reached food
                if let Ok((target, physics)) = ants.get(*actor) {
                    if let AntTarget::Food(food_entity) = target {
                        if let Ok((_, _, food_transform)) = food_sources.get(*food_entity) {
                            let distance = physics.position.distance(food_transform.translation.truncate());
                            if distance < 25.0 {
                                *action_state = ActionState::Success;
                                debug!("Ant {:?} reached food source {:?} (distance: {:.1})", actor, food_entity, distance);
                            }
                        } else {
                            *action_state = ActionState::Failure;
                        }
                    }
                }
            }
            ActionState::Cancelled => {
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

/// System that handles food collection behavior (simplified)
pub fn collect_food_action_system(
    mut ants: Query<(&mut CarriedResources, &AntPhysics, &mut AntTarget, &mut AntState)>,
    mut food_sources: Query<(&mut FoodSourceProperties, &Transform), With<FoodSource>>,
    mut action_query: Query<(&Actor, &mut ActionState), With<CollectFoodAction>>,
) {
    for (Actor(actor), mut action_state) in action_query.iter_mut() {
        match *action_state {
            ActionState::Requested => {
                info!("CollectFoodAction requested for ant {:?}", actor);
                if let Ok((mut resources, physics, mut target, mut state)) = ants.get_mut(*actor) {
                    let mut collected = false;
                    
                    debug!("Ant {:?} attempting to collect food at position ({:.1}, {:.1})", 
                           actor, physics.position.x, physics.position.y);
                    
                    for (mut food, food_transform) in food_sources.iter_mut() {
                        let food_pos = food_transform.translation.truncate();
                        let distance = physics.position.distance(food_pos);
                        
                        debug!("  Checking food at ({:.1}, {:.1}), amount: {:.1}, distance: {:.1}", 
                               food_pos.x, food_pos.y, food.amount, distance);
                        
                        if distance < 25.0 && food.amount > 0.0 {
                            let collect_amount = (food.amount * 0.1).min(resources.capacity - resources.current_weight);
                            
                            if collect_amount > 0.0 {
                                food.amount -= collect_amount;
                                *resources.resources.entry(food.food_type.clone()).or_insert(0.0) += collect_amount;
                                resources.current_weight += collect_amount;
                                *target = AntTarget::None;
                                
                                // Transition to CarryingFood state after successful collection
                                *state = AntState::CarryingFood;
                                
                                collected = true;
                                info!("🍎 Ant {:?} collected {:.2} {} food and transitioned to CarryingFood state", 
                                      actor, collect_amount, food.food_type);
                                break;
                            }
                        }
                    }
                    
                    if !collected {
                        debug!("Ant {:?} could not collect food (no food in range)", actor);
                    }
                    
                    *action_state = if collected { ActionState::Success } else { ActionState::Failure };
                } else {
                    warn!("Could not get ant components for {:?}", actor);
                    *action_state = ActionState::Failure;
                }
            }
            ActionState::Cancelled => {
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

/// System that handles returning to colony with food
pub fn return_to_colony_action_system(
    mut ants: Query<(&mut AntTarget, &AntPhysics, &CarriedResources)>,
    colonies: Query<(Entity, &ColonyProperties, &Transform), With<Colony>>,
    mut action_query: Query<(&Actor, &mut ActionState), With<ReturnToColonyAction>>,
) {
    for (Actor(actor), mut action_state) in action_query.iter_mut() {
        match *action_state {
            ActionState::Requested => {
                if let Ok((mut target, physics, resources)) = ants.get_mut(*actor) {
                    // Only return to colony if carrying food
                    if !resources.resources.is_empty() {
                        // Find nearest colony
                        let mut nearest_colony = None;
                        let mut nearest_distance = f32::INFINITY;
                        
                        for (colony_entity, colony_props, colony_transform) in colonies.iter() {
                            let distance = physics.position.distance(colony_transform.translation.truncate());
                            if distance < nearest_distance {
                                nearest_distance = distance;
                                nearest_colony = Some(colony_entity);
                            }
                        }
                        
                        if let Some(colony_entity) = nearest_colony {
                            *target = AntTarget::Colony(colony_entity);
                            *action_state = ActionState::Executing;
                            debug!("Ant {:?} returning to colony {:?} with food (weight: {:.2})", 
                                   actor, colony_entity, resources.current_weight);
                        } else {
                            *action_state = ActionState::Failure;
                        }
                    } else {
                        *action_state = ActionState::Failure;
                    }
                }
            }
            ActionState::Executing => {
                // Check if ant reached colony
                if let Ok((target, physics, _)) = ants.get(*actor) {
                    if let AntTarget::Colony(colony_entity) = target {
                        if let Ok((_, colony_props, colony_transform)) = colonies.get(*colony_entity) {
                            let distance = physics.position.distance(colony_transform.translation.truncate());
                            if distance < colony_props.radius + 10.0 {
                                *action_state = ActionState::Success;
                                debug!("Ant {:?} reached colony, ready to deposit food", actor);
                            }
                        } else {
                            *action_state = ActionState::Failure;
                        }
                    }
                }
            }
            ActionState::Cancelled => {
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

/// System that handles depositing food at colony
pub fn deposit_food_action_system(
    mut ants: Query<(&mut CarriedResources, &AntPhysics, &mut AntTarget, &mut AntState)>,
    mut colonies: Query<(&mut ColonyResources, &ColonyProperties, &Transform), With<Colony>>,
    mut action_query: Query<(&Actor, &mut ActionState), With<DepositFoodAction>>,
    mut simulation_stats: ResMut<SimulationStats>,
) {
    for (Actor(actor), mut action_state) in action_query.iter_mut() {
        match *action_state {
            ActionState::Requested => {
                if let Ok((mut resources, physics, mut target, mut state)) = ants.get_mut(*actor) {
                    let mut deposited = false;
                    
                    // Find nearby colony to deposit food
                    for (mut colony_resources, colony_props, colony_transform) in colonies.iter_mut() {
                        let distance = physics.position.distance(colony_transform.translation.truncate());
                        
                        if distance < colony_props.radius + 10.0 && !resources.resources.is_empty() {
                            // Transfer all carried resources to colony
                            let total_deposited = resources.current_weight;
                            
                            for (food_type, amount) in resources.resources.drain() {
                                *colony_resources.resources.entry(food_type).or_insert(0.0) += amount;
                                simulation_stats.total_food_collected += amount;
                            }
                            
                            resources.current_weight = 0.0;
                            *target = AntTarget::None;
                            
                            // Transition back to wandering (role determines next behavior)
                            *state = AntState::Wandering;
                            
                            deposited = true;
                            info!("Ant {:?} deposited {:.2} food at colony", actor, total_deposited);
                            break;
                        }
                    }
                    
                    *action_state = if deposited { ActionState::Success } else { ActionState::Failure };
                }
            }
            ActionState::Cancelled => {
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

/// System for scout ants to mark found food sources with pheromones
pub fn mark_food_source_action_system(
    mut commands: Commands,
    mut ants: Query<(&AntPhysics, &mut AntTarget, &mut AntState, &mut AntMemory, &AntType)>,
    food_sources: Query<(Entity, &FoodSourceProperties, &Transform), With<FoodSource>>,
    mut action_query: Query<(&Actor, &mut ActionState), With<MarkFoodSourceAction>>,
    simulation_state: Res<SimulationState>,
) {
    for (Actor(actor), mut action_state) in action_query.iter_mut() {
        match *action_state {
            ActionState::Requested => {
                if let Ok((physics, mut target, mut state, mut memory, ant_type)) = ants.get_mut(*actor) {
                    if ant_type.role != "scout" {
                        *action_state = ActionState::Failure;
                        continue;
                    }
                    
                    let mut marked = false;
                    
                    // Check if scout is near an undiscovered food source
                    for (food_entity, food_props, food_transform) in food_sources.iter() {
                        let distance = physics.position.distance(food_transform.translation.truncate());
                        
                        if distance < 25.0 && food_props.amount > 0.0 {
                            // Check if this food source is already known
                            if !memory.known_food_sources.contains(&food_entity) {
                                // Mark food source as discovered
                                memory.known_food_sources.push(food_entity);
                                
                                // Create strong food pheromone at food source location
                                commands.spawn((
                                    PheromoneTrail,
                                    PheromoneProperties {
                                        trail_type: PheromoneType::Food,
                                        strength: 200.0, // Very strong initial marker
                                        max_strength: 200.0,
                                        decay_rate: 0.02, // Slow decay for food markers
                                        expires_at: simulation_state.current_tick + 30000,
                                        source_ant: Some(*actor),
                                        target_food: Some(food_entity),
                                    },
                                    Transform::from_translation(food_transform.translation),
                                ));
                                
                                // Set target to return to colony
                                *target = AntTarget::None;
                                *state = AntState::Following; // Scout returns following its own trail
                                
                                marked = true;
                                info!("Scout ant {:?} discovered and marked food source {:?}", actor, food_entity);
                                break;
                            }
                        }
                    }
                    
                    *action_state = if marked { ActionState::Success } else { ActionState::Failure };
                }
            }
            ActionState::Cancelled => {
                *action_state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

// ============================================================================
// PLUGIN DEFINITION
// ============================================================================

pub struct AntBehaviorPlugin;

impl Plugin for AntBehaviorPlugin {
    fn build(&self, app: &mut App) {
        info!("🧠 Registering AntBehaviorPlugin with Big-Brain AI systems");
        
        // Note: BigBrainPlugin is already registered in simulation.rs
        
        // Add scorer systems (using Update schedule to match BigBrainPlugin)
        app.add_systems(Update, (
            scout_scorer_system,
            carrying_food_scorer_system,
            near_food_scorer_system,
            near_colony_scorer_system,
            pheromone_strength_scorer_system,
            exploration_urge_scorer_system,
            needs_rest_scorer_system,
            stuck_scorer_system,
        ).in_set(BigBrainSet::Scorers));
        
        // Add action systems (using Update schedule to match BigBrainPlugin)
        app.add_systems(Update, (
            wander_action_system,
            seek_food_action_system,
            collect_food_action_system,
            return_to_colony_action_system,
            deposit_food_action_system,
            mark_food_source_action_system,
        ).in_set(BigBrainSet::Actions));
        
        // Add the basic movement system that actually moves the ants
        app.add_systems(Update, crate::managers::ant_spawn::basic_ant_movement_system);
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