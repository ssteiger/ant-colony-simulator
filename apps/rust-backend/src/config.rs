use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct SimConfig {
    pub simulation_id: i32,
    pub world_width: f32,
    pub world_height: f32,
    pub tick_rate: f32,

    pub ant_max_speed: f32,
    pub ant_turn_rate: f32,
    pub ant_wander_strength: f32,
    pub ant_sensor_distance: f32,
    pub ant_sensor_angle: f32,
    pub ant_pickup_radius: f32,
    pub ant_detection_radius: f32,

    pub pheromone_cell_size: f32,
    pub pheromone_food_deposit: f32,
    pub pheromone_evaporation: f32,
    pub pheromone_diffusion_rate: f32,
    pub pheromone_diffusion_interval: u64,

    pub initial_ant_count: usize,
    pub max_ants: usize,
    pub colony_radius: f32,

    pub food_source_count: usize,
    pub food_per_source: f32,
    pub food_min_distance_from_colony: f32,

    pub boundary_margin: f32,

    /// Colony food consumed per living ant per tick (trophallaxis upkeep).
    pub colony_upkeep_per_ant: f32,
    /// Vitality regained per tick while the colony can feed the ant.
    pub ant_feed_recovery: f32,
    /// Vitality lost per tick when the colony has no food to spare.
    pub ant_starve_damage: f32,
    /// Base worker lifespan in ticks; ants die of old age past this.
    pub ant_lifespan_ticks: u64,
    /// Per-ant random spread added to the base lifespan.
    pub ant_lifespan_variation: u64,
    pub colony_spawn_cost: f32,
    pub colony_spawn_interval: u64,
    pub colony_spawn_batch: usize,
    pub levy_cooldown_ticks: u32,
    pub levy_speed_boost: f32,
    pub scout_wander_boost: f32,
    pub soldier_patrol_radius: f32,

    pub terrain_cell_size: f32,
    pub terrain_seed: u64,
    pub terrain_density: f32,
    pub terrain_smooth_iterations: u32,
    pub wall_probe_distance: f32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            simulation_id: 1,
            world_width: 4000.0,
            world_height: 3000.0,
            tick_rate: 60.0,

            ant_max_speed: 80.0,
            ant_turn_rate: 4.0,
            ant_wander_strength: 0.6,
            ant_sensor_distance: 24.0,
            ant_sensor_angle: 0.5,
            ant_pickup_radius: 10.0,
            ant_detection_radius: 50.0,

            pheromone_cell_size: 8.0,
            pheromone_food_deposit: 0.03,
            pheromone_evaporation: 0.996,
            pheromone_diffusion_rate: 0.08,
            pheromone_diffusion_interval: 3,

            initial_ant_count: 5_000,
            max_ants: 50_000,
            colony_radius: 50.0,

            food_source_count: 48,
            food_per_source: 2_500.0,
            food_min_distance_from_colony: 500.0,

            boundary_margin: 40.0,

            // Upkeep is negligible for a well-fed colony (so ants normally die
            // of old age) but becomes fatal if foraging income dries up,
            // letting a starved colony collapse.
            colony_upkeep_per_ant: 0.0003,
            ant_feed_recovery: 0.05,
            ant_starve_damage: 0.02,
            ant_lifespan_ticks: 27_000,
            ant_lifespan_variation: 9_000,
            colony_spawn_cost: 2.0,
            colony_spawn_interval: 30,
            colony_spawn_batch: 8,
            levy_cooldown_ticks: 180,
            levy_speed_boost: 3.0,
            scout_wander_boost: 1.6,
            soldier_patrol_radius: 120.0,

            terrain_cell_size: 8.0,
            terrain_seed: 42,
            terrain_density: 0.32,
            terrain_smooth_iterations: 5,
            wall_probe_distance: 14.0,
        }
    }
}

/// Optional per-simulation overrides stored in the `simulations.config` jsonb column.
#[derive(Debug, Default, Deserialize)]
pub struct SimOverrides {
    pub seed: Option<u64>,
    pub terrain_density: Option<f32>,
    pub initial_ants: Option<usize>,
    pub max_ants: Option<usize>,
    pub food_sources: Option<usize>,
    pub food_per_source: Option<f32>,
}

impl SimConfig {
    /// Build a config from a `simulations` table row.
    pub fn from_row(id: i32, world_width: i32, world_height: i32, config_json: &serde_json::Value) -> Self {
        let mut cfg = Self {
            simulation_id: id,
            world_width: world_width as f32,
            world_height: world_height as f32,
            ..Self::default()
        };

        let overrides: SimOverrides =
            serde_json::from_value(config_json.clone()).unwrap_or_default();

        if let Some(seed) = overrides.seed {
            cfg.terrain_seed = seed;
        } else {
            cfg.terrain_seed = id as u64;
        }
        if let Some(d) = overrides.terrain_density {
            cfg.terrain_density = d.clamp(0.0, 0.6);
        }
        if let Some(n) = overrides.initial_ants {
            cfg.initial_ant_count = n.min(cfg.max_ants);
        }
        if let Some(n) = overrides.max_ants {
            cfg.max_ants = n.clamp(100, 200_000);
            cfg.initial_ant_count = cfg.initial_ant_count.min(cfg.max_ants);
        }
        if let Some(n) = overrides.food_sources {
            cfg.food_source_count = n.clamp(1, 500);
        }
        if let Some(f) = overrides.food_per_source {
            cfg.food_per_source = f.max(10.0);
        }

        // scale food spacing down for small worlds
        let max_dist = (cfg.world_width.min(cfg.world_height)) * 0.35;
        cfg.food_min_distance_from_colony = cfg.food_min_distance_from_colony.min(max_dist);

        cfg
    }
}
