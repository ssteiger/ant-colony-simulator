use crate::models::*;
use bevy::prelude::*;
use big_brain::prelude::*;
use tracing::{debug, info, trace};

// ============================================================================
// BIG-BRAIN SCORERS - Decision Inputs for Ant AI
// ============================================================================

/// Scorer that evaluates how hungry/low-energy an ant is
#[derive(Component, Debug, Clone, ScorerBuilder)]
pub struct HungryScorer;

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

// ============================================================================
// SCORER SYSTEMS - Evaluate World State
// ============================================================================

/// System that scores how hungry an ant is based on energy levels
pub fn hungry_scorer_system(
    ant_healths: Query<&AntHealth>,
    mut scorers: Query<(&Actor, &mut Score), With<HungryScorer>>,
) {
    for (Actor(actor), mut score) in scorers.iter_mut() {
        if let Ok(health) = ant_healths.get(*actor) {
            // Convert energy to hunger score (1.0 = very hungry, 0.0 = not hungry)
            let hunger_score = 1.0 - (health.energy / health.max_energy).clamp(0.0, 1.0);
            score.set(hunger_score);
            
            if hunger_score > 0.7 {
                trace!("Ant {:?} is hungry (score: {:.2}, energy: {:.1}/{:.1})", 
                       actor, hunger_score, health.energy, health.max_energy);
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
            
            for (food_props, food_transform) in food_sources.iter() {
                if food_props.amount > 0.0 {
                    let distance = physics.position.distance(food_transform.translation.truncate());
                    
                    // Score based on proximity and food amount
                    let proximity_score = if distance < 150.0 {
                        let base_score = 1.0 - (distance / 150.0);
                        // Boost score based on food amount
                        base_score * (food_props.amount / 100.0).clamp(0.1, 1.0)
                    } else {
                        0.0
                    };
                    
                    best_score = best_score.max(proximity_score);
                }
            }
            
            score.set(best_score);
            
            if best_score > 0.3 {
                trace!("Ant {:?} near food (score: {:.2})", actor, best_score);
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

/// System that scores pheromone trail strength (simplified for now)
pub fn pheromone_strength_scorer_system(
    _ant_physics: Query<&AntPhysics>,
    _pheromones: Query<&PheromoneProperties, With<PheromoneTrail>>,
    mut scorers: Query<(&Actor, &mut Score), With<PheromoneStrengthScorer>>,
) {
    for (Actor(_actor), mut score) in scorers.iter_mut() {
        // Simplified for now since PheromoneProperties doesn't have position
        let pheromone_score = 0.0;
        score.set(pheromone_score);
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

/// System that scores if ant needs rest (very low energy)
pub fn needs_rest_scorer_system(
    ant_healths: Query<&AntHealth>,
    mut scorers: Query<(&Actor, &mut Score), With<NeedsRestScorer>>,
) {
    for (Actor(actor), mut score) in scorers.iter_mut() {
        if let Ok(health) = ant_healths.get(*actor) {
            // Only need rest if energy is critically low
            let rest_score = if health.energy < 10.0 {
                1.0 - (health.energy / 10.0)
            } else {
                0.0
            };
            
            score.set(rest_score);
            
            if rest_score > 0.0 {
                debug!("Ant {:?} needs rest (score: {:.2}, energy: {:.1})", actor, rest_score, health.energy);
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
                            if distance < 15.0 {
                                *action_state = ActionState::Success;
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
    mut ants: Query<(&mut CarriedResources, &AntPhysics, &mut AntTarget)>,
    mut food_sources: Query<(&mut FoodSourceProperties, &Transform), With<FoodSource>>,
    mut action_query: Query<(&Actor, &mut ActionState), With<CollectFoodAction>>,
) {
    for (Actor(actor), mut action_state) in action_query.iter_mut() {
        match *action_state {
            ActionState::Requested => {
                if let Ok((mut resources, physics, mut target)) = ants.get_mut(*actor) {
                    let mut collected = false;
                    
                    for (mut food, food_transform) in food_sources.iter_mut() {
                        let distance = physics.position.distance(food_transform.translation.truncate());
                        
                        if distance < 15.0 && food.amount > 0.0 {
                            let collect_amount = (food.amount * 0.1).min(resources.capacity - resources.current_weight);
                            
                            if collect_amount > 0.0 {
                                food.amount -= collect_amount;
                                *resources.resources.entry(food.food_type.clone()).or_insert(0.0) += collect_amount;
                                resources.current_weight += collect_amount;
                                *target = AntTarget::None;
                                collected = true;
                                info!("Ant {:?} collected {:.2} food", actor, collect_amount);
                                break;
                            }
                        }
                    }
                    
                    *action_state = if collected { ActionState::Success } else { ActionState::Failure };
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
            hungry_scorer_system,
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
        ).in_set(BigBrainSet::Actions));
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