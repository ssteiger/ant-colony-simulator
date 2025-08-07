use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin};
use big_brain::prelude::*;
use crate::models::*;
use crate::managers::*;
use crate::database::DatabaseManager;
use crate::utils::{DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT, world_center, get_world_bounds};
use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::runtime::Runtime;

/// Main Bevy app for the ant colony simulation
pub struct AntColonySimulator {
    app: App,
    db: Option<Arc<DatabaseManager>>,
    simulation_id: i32,
    runtime: Runtime,
}

impl AntColonySimulator {
    pub fn new(
        db_pool: sqlx::PgPool,
        simulation_id: i32,
    ) -> Result<Self> {
        let runtime = Runtime::new()?;
        let db = Arc::new(DatabaseManager::new(db_pool));
        
        // Load simulation data from database
        let simulation = runtime.block_on(db.load_simulation(simulation_id))?;
        
        // Use window dimensions as world bounds for full window coverage
        let (world_width, world_height) = get_world_bounds(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);
        let world_bounds = WorldBounds::centered(world_width, world_height);

        // Create Bevy app
        let mut app = App::new();
        
        // Add default plugins with window configuration
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ant Colony Simulator".to_string(),
                resolution: (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }));
        
        // Set custom clear color to match background
        app.insert_resource(ClearColor(Color::WHITE));

        // Add Big Brain plugin
        app.add_plugins(BigBrainPlugin::new(Update));

        // Add simulation resources
        app.insert_resource(SimulationState {
            simulation_id,
            current_tick: simulation.current_tick.unwrap_or(0),
            world_bounds: world_bounds.clone(),
            is_running: true,
            simulation_speed: simulation.simulation_speed.unwrap_or(1) as f32,
        });

        app.insert_resource(SimulationStats {
            total_ants: 0,
            active_colonies: 0,
            total_food_collected: 0.0,
            pheromone_trail_count: 0,
            current_tick: 0,
        });

        app.insert_resource(world_bounds);

        // Add all simulation plugins
        app.add_plugins((
            AntBehaviorPlugin,
            ColonyPlugin,
            EnvironmentPlugin,
            PheromonePlugin,
            RenderingPlugin,
        ));

        // Add custom systems
        app.add_systems(Update, (
            simulation_tick_system,
            database_sync_system,
            update_simulation_stats,
        ));

        // Initialize simulation with database data
        Self::load_initial_data(&mut app, &db, simulation_id, &runtime)?;

        Ok(Self {
            app,
            db: Some(db),
            simulation_id,
            runtime,
        })
    }

    pub fn new_test() -> Result<Self> {
        let runtime = Runtime::new()?;
        
        // Use window dimensions as world bounds for full window coverage
        let (world_width, world_height) = get_world_bounds(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);
        let world_bounds = WorldBounds::centered(world_width, world_height);

        // Create Bevy app
        let mut app = App::new();
        
        // Add default plugins with window configuration
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ant Colony Simulator (Test Mode)".to_string(),
                resolution: (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }));
        
        // Set custom clear color to match background
        app.insert_resource(ClearColor(Color::WHITE));

        // Add Big Brain plugin
        app.add_plugins(BigBrainPlugin::new(Update));

        // Add simulation resources
        app.insert_resource(SimulationState {
            simulation_id: 1,
            current_tick: 0,
            world_bounds: world_bounds.clone(),
            is_running: true,
            simulation_speed: 1.0,
        });

        app.insert_resource(SimulationStats {
            total_ants: 0,
            active_colonies: 0,
            total_food_collected: 0.0,
            pheromone_trail_count: 0,
            current_tick: 0,
        });

        app.insert_resource(world_bounds);

        // Add all simulation plugins
        app.add_plugins((
            AntBehaviorPlugin,
            ColonyPlugin,
            EnvironmentPlugin,
            PheromonePlugin,
            RenderingPlugin,
        ));

        // Add custom systems (without database sync for test)
        app.add_systems(Update, (
            simulation_tick_system,
            update_simulation_stats,
        ));

        // Add some test entities
        Self::add_test_entities(&mut app);

        Ok(Self {
            app,
            db: None, // No database in test mode
            simulation_id: 1,
            runtime,
        })
    }

    fn add_test_entities(app: &mut App) {
        // Place entities relative to window center (0,0)
        let (center_x, center_y) = world_center(); // This is now (0,0)
        
        // Add a test colony at the center of the screen
        app.world.spawn((
            Colony,
            ColonyProperties {
                name: "Test Colony".to_string(),
                center: Vec2::new(center_x, center_y),
                radius: 50.0,
                population: 100,
                max_population: 1000,
                color_hue: 0.0,
                territory_radius: 1000.0,
                aggression_level: 0.5,
            },
            ColonyResources {
                resources: HashMap::new(),
                storage_capacity: HashMap::new(),
            },
            ColonyNest {
                level: 1,
                max_level: 10,
                upgrade_cost: HashMap::new(),
            },
            Transform::from_translation(Vec3::new(center_x, center_y, 0.0)),
        ));

        // Add test food sources around the center in a visible pattern
        let food_positions = [
            (150.0, 150.0),   // Top-right
            (-150.0, 150.0),  // Top-left
            (150.0, -150.0),  // Bottom-right
            (-150.0, -150.0), // Bottom-left
            (200.0, 0.0),     // Right
            (-200.0, 0.0),    // Left
        ];

        for (i, (offset_x, offset_y)) in food_positions.iter().enumerate() {
            let food_type = match i % 4 {
                0 => "berries",
                1 => "leaves", 
                2 => "seeds",
                _ => "nuts",
            };
            
            app.world.spawn((
                FoodSource,
                FoodSourceProperties {
                    food_type: food_type.to_string(),
                    amount: 100.0,
                    max_amount: 100.0,
                    regeneration_rate: 0.1,
                    is_renewable: true,
                    nutritional_value: 10.0,
                    spoilage_rate: 0.01,
                    discovery_difficulty: 0.5,
                },
                Transform::from_translation(Vec3::new(center_x + offset_x, center_y + offset_y, 0.0)),
            ));
        }

        // Add test ants scattered around the center
        for i in 0..5 {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / 5.0;
            let radius = 75.0;
            let x = center_x + angle.cos() * radius;
            let y = center_y + angle.sin() * radius;
            
            app.world.spawn((
                Ant,
                AntPhysics {
                    position: Vec2::new(x, y),
                    velocity: Vec2::ZERO,
                    max_speed: 50.0,
                    acceleration: 100.0,
                    rotation: angle,
                    rotation_speed: 2.0,
                    desired_direction: Vec2::new(angle.cos(), angle.sin()),
                    momentum: 0.95,
                    last_positions: Vec::new(),
                    turn_smoothness: 3.0,
                    wander_angle: angle,
                    wander_change: 0.3,
                    obstacle_avoidance_force: Vec2::ZERO,
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
                    capacity: 50.0,
                    current_weight: 0.0,
                },
                AntTarget::None,
                AntMemory {
                    known_food_sources: Vec::new(),
                    known_colonies: Vec::new(),
                    last_food_source: None,
                    last_action_tick: 0,
                    pheromone_sensitivity: 0.5,
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
                    carrying_capacity: 50.0,
                    color_hue: 0.0,
                    special_abilities: Vec::new(),
                },
                Transform::from_translation(Vec3::new(x, y, 0.0)),
            ));
        }
    }

    fn load_initial_data(
        app: &mut App,
        db: &Arc<DatabaseManager>,
        simulation_id: i32,
        runtime: &Runtime,
    ) -> Result<()> {
        // Load colonies
        let colonies = runtime.block_on(db.load_colonies(simulation_id))?;
        for colony in colonies {
            app.world.spawn((
                Colony,
                ColonyProperties {
                    name: colony.name,
                    center: Vec2::new(colony.center_x as f32, colony.center_y as f32),
                    radius: colony.radius as f32,
                    population: colony.population,
                    max_population: 100,
                    color_hue: colony.color_hue as f32,
                    territory_radius: colony.territory_radius as f32,
                    aggression_level: colony.aggression_level as f32,
                },
                ColonyResources {
                    resources: Self::parse_resources(&colony.resources),
                    storage_capacity: HashMap::new(),
                },
                ColonyNest {
                    level: colony.nest_level,
                    max_level: 10,
                    upgrade_cost: HashMap::new(),
                },
                Transform::from_translation(Vec3::new(
                    colony.center_x as f32,
                    colony.center_y as f32,
                    0.0,
                )),
            ));
        }

        // Load food sources
        let food_sources = runtime.block_on(db.load_food_sources(simulation_id))?;
        for food in food_sources {
            app.world.spawn((
                FoodSource,
                FoodSourceProperties {
                    food_type: food.food_type,
                    amount: food.amount as f32,
                    max_amount: food.max_amount as f32,
                    regeneration_rate: food.regeneration_rate.unwrap_or(0) as f32 / 100.0,
                    is_renewable: food.is_renewable.unwrap_or(false),
                    nutritional_value: food.nutritional_value as f32,
                    spoilage_rate: food.spoilage_rate.unwrap_or(0) as f32 / 100.0,
                    discovery_difficulty: food.discovery_difficulty.unwrap_or(50) as f32 / 100.0,
                },
                Transform::from_translation(Vec3::new(
                    food.position_x as f32,
                    food.position_y as f32,
                    0.0,
                )),
            ));
        }

        // Load ants
        let ants = runtime.block_on(db.load_ants(simulation_id))?;
        for ant in ants {
            app.world.spawn((
                Ant,
                AntPhysics {
                    position: Vec2::new(ant.position_x as f32, ant.position_y as f32),
                    velocity: Vec2::ZERO,
                    max_speed: 50.0,
                    acceleration: 100.0,
                    rotation: ant.angle as f32,
                    rotation_speed: 2.0,
                    desired_direction: Vec2::new(1.0, 0.0),
                    momentum: 0.95,
                    last_positions: Vec::new(),
                    turn_smoothness: 3.0,
                    wander_angle: ant.angle as f32,
                    wander_change: 0.3,
                    obstacle_avoidance_force: Vec2::ZERO,
                },
                AntHealth {
                    health: ant.health as f32,
                    max_health: 100.0,
                    energy: ant.energy as f32,
                    max_energy: 100.0,
                    age_ticks: ant.age_ticks as i64,
                    lifespan_ticks: 10000,
                },
                AntState::Wandering,
                CarriedResources {
                    resources: HashMap::new(),
                    capacity: 50.0,
                    current_weight: 0.0,
                },
                AntTarget::None,
                AntMemory {
                    known_food_sources: Vec::new(),
                    known_colonies: Vec::new(),
                    last_food_source: None,
                    last_action_tick: 0,
                    pheromone_sensitivity: 0.5,
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
                    carrying_capacity: 50.0,
                    color_hue: 0.0,
                    special_abilities: Vec::new(),
                },
                Transform::from_translation(Vec3::new(
                    ant.position_x as f32,
                    ant.position_y as f32,
                    0.0,
                )),
            ));
        }

        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        tracing::info!("🎮 Starting Bevy ant colony simulation...");
        
        // Run the Bevy app
        self.app.run();
        
        Ok(())
    }

    pub fn stop(&mut self) {
        tracing::info!("🛑 Stopping simulation...");
        if let Some(mut simulation_state) = self.app.world.get_resource_mut::<SimulationState>() {
            simulation_state.is_running = false;
        }
    }

    // Utility functions for data conversion
    fn parse_resources(resources_json: &serde_json::Value) -> HashMap<String, f32> {
        let mut resources = HashMap::new();
        
        if let Some(obj) = resources_json.as_object() {
            for (key, value) in obj {
                if let Some(amount) = value.as_f64() {
                    resources.insert(key.clone(), amount as f32);
                }
            }
        }
        
        resources
    }
}

// ============================================================================
// SIMULATION SYSTEMS
// ============================================================================

/// System to update simulation tick counter
fn simulation_tick_system(
    mut simulation_state: ResMut<SimulationState>,
    mut stats: ResMut<SimulationStats>,
) {
    simulation_state.current_tick += 1;
    stats.current_tick = simulation_state.current_tick;
}

/// System to sync simulation state with database
fn database_sync_system(
    simulation_state: Res<SimulationState>,
    ants: Query<(&AntPhysics, &AntHealth, &AntState), With<Ant>>,
    colonies: Query<(&ColonyProperties, &ColonyResources), With<Colony>>,
    food_sources: Query<(&FoodSourceProperties, &Transform), With<FoodSource>>,
) {
    // This would sync the current state to the database
    // Implementation depends on the database manager
}

/// System to update simulation stats based on the current entities in the world
fn update_simulation_stats(
    mut stats: ResMut<SimulationStats>,
    ants: Query<(&AntPhysics, &AntHealth, &AntState), With<Ant>>,
    colonies: Query<(&ColonyProperties, &ColonyResources), With<Colony>>,
    food_sources: Query<(&FoodSourceProperties, &Transform), With<FoodSource>>,
    pheromones: Query<&PheromoneProperties, With<PheromoneTrail>>,
) {
    // Count total ants
    stats.total_ants = ants.iter().count() as i32;
    
    // Count active colonies
    stats.active_colonies = colonies.iter().count() as i32;
    
    // Calculate total food collected from colonies
    let mut total_food = 0.0;
    for (_, colony_resources) in colonies.iter() {
        for (_, amount) in &colony_resources.resources {
            total_food += amount;
        }
    }
    stats.total_food_collected = total_food;
    
    // Count active pheromone trails
    stats.pheromone_trail_count = pheromones.iter().count() as i32;
}

// ============================================================================
// PLUGIN
// ============================================================================

pub struct AntColonySimulationPlugin;

impl Plugin for AntColonySimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            simulation_tick_system,
            database_sync_system,
            update_simulation_stats,
        ));
    }
} 