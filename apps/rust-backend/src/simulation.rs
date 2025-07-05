use bevy::prelude::*;
use big_brain::prelude::*;
use crate::models::*;
use crate::managers::*;
use crate::database::DatabaseManager;
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
        
        let world_bounds = WorldBounds {
            width: simulation.world_width as f32,
            height: simulation.world_height as f32,
        };

        // Create Bevy app
        let mut app = App::new();
        
        // Add default plugins
        app.add_plugins(DefaultPlugins);
        
        // Set custom clear color to match background
        app.insert_resource(ClearColor(Color::rgb(0.2, 0.4, 0.1)));

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
        
        let world_bounds = WorldBounds {
            width: 1000.0,
            height: 1000.0,
        };

        // Create Bevy app
        let mut app = App::new();
        
        // Add default plugins
        app.add_plugins(DefaultPlugins);
        
        // Set custom clear color to match background
        app.insert_resource(ClearColor(Color::rgb(0.2, 0.4, 0.1)));

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
        // Add a test colony
        app.world.spawn((
            Colony,
            ColonyProperties {
                name: "Test Colony".to_string(),
                center: Vec2::new(500.0, 500.0),
                radius: 50.0,
                population: 10,
                max_population: 100,
                color_hue: 0.0,
                territory_radius: 200.0,
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
            Transform::from_translation(Vec3::new(500.0, 500.0, 0.0)),
        ));

        // Add a test food source
        app.world.spawn((
            FoodSource,
            FoodSourceProperties {
                food_type: "berries".to_string(),
                amount: 100.0,
                max_amount: 100.0,
                regeneration_rate: 0.1,
                is_renewable: true,
                nutritional_value: 10.0,
                spoilage_rate: 0.01,
                discovery_difficulty: 0.5,
            },
            Transform::from_translation(Vec3::new(200.0, 200.0, 0.0)),
        ));

        // Add a test ant
        app.world.spawn((
            Ant,
            AntPhysics {
                position: Vec2::new(500.0, 500.0),
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
            Transform::from_translation(Vec3::new(500.0, 500.0, 0.0)),
        ));
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
    
    // Count pheromone trails (this would need to be implemented when pheromones are added)
    stats.pheromone_trail_count = 0; // Placeholder
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