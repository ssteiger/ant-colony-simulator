#[derive(Clone, Debug)]
pub struct SimConfig {
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
    pub pheromone_home_deposit: f32,
    pub pheromone_food_deposit: f32,
    pub pheromone_evaporation: f32,
    pub pheromone_diffusion_rate: f32,
    pub pheromone_diffusion_interval: u64,

    pub spatial_cell_size: f32,

    pub initial_ant_count: usize,
    pub colony_radius: f32,

    pub food_source_count: usize,
    pub food_per_source: f32,
    pub food_min_distance_from_colony: f32,

    pub boundary_margin: f32,

    pub ant_energy_decay_per_tick: f32,
    pub ant_energy_food_restore: f32,
    pub colony_spawn_cost: f32,
    pub colony_spawn_interval: u64,
    pub levy_cooldown_ticks: u32,
    pub levy_speed_boost: f32,
    pub scout_wander_boost: f32,
    pub soldier_patrol_radius: f32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            world_width: 1200.0,
            world_height: 800.0,
            tick_rate: 60.0,

            ant_max_speed: 80.0,
            ant_turn_rate: 4.0,
            ant_wander_strength: 0.6,
            ant_sensor_distance: 20.0,
            ant_sensor_angle: 0.5,
            ant_pickup_radius: 8.0,
            ant_detection_radius: 40.0,

            pheromone_cell_size: 4.0,
            pheromone_home_deposit: 0.015,
            pheromone_food_deposit: 0.04,
            pheromone_evaporation: 0.997,
            pheromone_diffusion_rate: 0.08,
            pheromone_diffusion_interval: 3,

            spatial_cell_size: 30.0,

            initial_ant_count: 200,
            colony_radius: 25.0,

            food_source_count: 10,
            food_per_source: 500.0,
            food_min_distance_from_colony: 150.0,

            boundary_margin: 40.0,

            ant_energy_decay_per_tick: 0.012,
            ant_energy_food_restore: 40.0,
            colony_spawn_cost: 8.0,
            colony_spawn_interval: 120,
            levy_cooldown_ticks: 180,
            levy_speed_boost: 3.0,
            scout_wander_boost: 1.6,
            soldier_patrol_radius: 80.0,
        }
    }
}
