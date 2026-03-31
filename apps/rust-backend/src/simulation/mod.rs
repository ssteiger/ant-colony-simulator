pub mod ant;
pub mod colony;
pub mod food;
pub mod pheromone;
pub mod snapshot;
pub mod spatial;
pub mod steering;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::config::SimConfig;
use ant::{speed_for_role, AntState, AntStorage, ROLE_SCOUT, ROLE_SOLDIER, ROLE_WORKER};
use colony::Colony;
use food::FoodSource;
use pheromone::{PheromoneField, PheromoneType};
use spatial::SpatialGrid;

pub struct SimulationState {
    pub config: SimConfig,
    pub ants: AntStorage,
    pub colonies: Vec<Colony>,
    pub food_sources: Vec<FoodSource>,
    pub pheromones: PheromoneField,
    pub spatial_grid: SpatialGrid,
    pub tick_count: u64,
    pub total_food_collected: f32,
    rng: SmallRng,
}

impl SimulationState {
    pub fn new(config: SimConfig) -> Self {
        let mut rng = SmallRng::from_entropy();

        let pheromones = PheromoneField::new(
            config.world_width,
            config.world_height,
            config.pheromone_cell_size,
        );
        let spatial_grid = SpatialGrid::new(config.spatial_cell_size);

        let cx = config.world_width / 2.0;
        let cy = config.world_height / 2.0;

        let colonies = vec![Colony {
            id: 0,
            x: cx,
            y: cy,
            radius: config.colony_radius,
            food_stored: 50.0,
            color_hue: 30,
        }];

        let mut food_sources = Vec::with_capacity(config.food_source_count);
        for i in 0..config.food_source_count {
            loop {
                let fx = rng.gen_range(50.0..config.world_width - 50.0);
                let fy = rng.gen_range(50.0..config.world_height - 50.0);
                let dx = fx - cx;
                let dy = fy - cy;
                if (dx * dx + dy * dy).sqrt() > config.food_min_distance_from_colony {
                    food_sources.push(FoodSource {
                        id: i as u32,
                        x: fx,
                        y: fy,
                        amount: config.food_per_source,
                        max_amount: config.food_per_source,
                    });
                    break;
                }
            }
        }

        // spawn initial ants with role distribution: 70% worker, 20% scout, 10% soldier
        let mut ants = AntStorage::new();
        for k in 0..config.initial_ant_count {
            let role = if k % 10 < 7 {
                ROLE_WORKER
            } else if k % 10 < 9 {
                ROLE_SCOUT
            } else {
                ROLE_SOLDIER
            };
            let angle: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
            let r: f32 = rng.gen_range(0.0..config.colony_radius * 0.8);
            let ax = cx + angle.cos() * r;
            let ay = cy + angle.sin() * r;
            let heading: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
            let spd = speed_for_role(role, config.ant_max_speed);
            ants.add(ax, ay, 0, role, spd, heading);
        }

        Self {
            config,
            ants,
            colonies,
            food_sources,
            pheromones,
            spatial_grid,
            tick_count: 0,
            total_food_collected: 0.0,
            rng,
        }
    }

    pub fn tick(&mut self) {
        let dt = 1.0 / self.config.tick_rate;
        self.tick_count += 1;

        self.spatial_grid
            .rebuild(&self.ants.pos_x, &self.ants.pos_y, self.ants.count);

        // update ants (iterate backwards so swap-remove is safe)
        let mut i = 0;
        while i < self.ants.count {
            self.update_ant(i, dt);

            // energy decay and death
            self.ants.energy[i] -= self.config.ant_energy_decay_per_tick;
            self.ants.age[i] += 1;

            if self.ants.energy[i] <= 0.0 {
                self.ants.remove(i);
                // don't increment i; the swapped-in element is now at i
            } else {
                i += 1;
            }
        }

        // colony spawning
        self.spawn_ants();

        self.pheromones.evaporate(self.config.pheromone_evaporation);

        if self.tick_count % self.config.pheromone_diffusion_interval == 0 {
            self.pheromones
                .diffuse(self.config.pheromone_diffusion_rate);
        }
    }

    fn spawn_ants(&mut self) {
        if self.tick_count % self.config.colony_spawn_interval != 0 {
            return;
        }
        for ci in 0..self.colonies.len() {
            let cost = self.config.colony_spawn_cost;
            if self.colonies[ci].food_stored < cost {
                continue;
            }
            self.colonies[ci].food_stored -= cost;

            let cx = self.colonies[ci].x;
            let cy = self.colonies[ci].y;
            let cid = self.colonies[ci].id;

            // pick role
            let r: f32 = self.rng.gen();
            let role = if r < 0.7 {
                ROLE_WORKER
            } else if r < 0.9 {
                ROLE_SCOUT
            } else {
                ROLE_SOLDIER
            };
            let spd = speed_for_role(role, self.config.ant_max_speed);

            let angle: f32 = self.rng.gen_range(0.0..std::f32::consts::TAU);
            let offset: f32 = self.rng.gen_range(0.0..self.colonies[ci].radius * 0.5);
            let ax = cx + angle.cos() * offset;
            let ay = cy + angle.sin() * offset;
            let heading: f32 = self.rng.gen_range(0.0..std::f32::consts::TAU);

            self.ants.add(ax, ay, cid, role, spd, heading);
        }
    }

    fn update_ant(&mut self, i: usize, dt: f32) {
        let x = self.ants.pos_x[i];
        let y = self.ants.pos_y[i];
        let heading = self.ants.heading[i];
        let state = self.ants.state[i];
        let role = self.ants.ant_type[i];

        // ── state transitions based on proximity ──────────────────────
        match state {
            AntState::Foraging => {
                let pickup_r_sq = self.config.ant_pickup_radius.powi(2);
                for j in 0..self.food_sources.len() {
                    if self.food_sources[j].amount <= 0.0 {
                        continue;
                    }
                    let dx = self.food_sources[j].x - x;
                    let dy = self.food_sources[j].y - y;
                    if dx * dx + dy * dy < pickup_r_sq {
                        self.food_sources[j].amount -= 1.0;
                        self.ants.cargo[i] = 1.0;
                        self.ants.state[i] = AntState::Returning;
                        break;
                    }
                }
            }
            AntState::Returning => {
                for j in 0..self.colonies.len() {
                    let dx = self.colonies[j].x - x;
                    let dy = self.colonies[j].y - y;
                    if dx * dx + dy * dy < self.colonies[j].radius.powi(2) {
                        self.colonies[j].food_stored += self.ants.cargo[i];
                        self.ants.cargo[i] = 0.0;
                        self.ants.state[i] = AntState::Foraging;
                        self.ants.home_vec_x[i] = 0.0;
                        self.ants.home_vec_y[i] = 0.0;
                        self.ants.heading[i] = self.rng.gen_range(0.0..std::f32::consts::TAU);
                        self.ants.wander_angle[i] = 0.0;
                        // restore some energy when depositing food
                        self.ants.energy[i] =
                            (self.ants.energy[i] + self.config.ant_energy_food_restore).min(100.0);
                        self.total_food_collected += 1.0;
                        break;
                    }
                }
            }
        }

        let state = self.ants.state[i];

        // ── levy flight (foraging wanderers only) ─────────────────────
        if state == AntState::Foraging && self.ants.levy_cooldown[i] == 0 {
            let r: f32 = self.rng.gen();
            if steering::levy_should_jump(r, 0) {
                self.ants.heading[i] = self.rng.gen_range(0.0..std::f32::consts::TAU);
                self.ants.levy_cooldown[i] = self.config.levy_cooldown_ticks;
            }
        }
        if self.ants.levy_cooldown[i] > 0 {
            self.ants.levy_cooldown[i] -= 1;
        }

        // speed multiplier for levy burst
        let levy_active = self.ants.levy_cooldown[i] > self.config.levy_cooldown_ticks.saturating_sub(30);
        let speed_mult = if levy_active {
            self.config.levy_speed_boost
        } else {
            1.0
        };
        let speed = self.ants.speed[i] * speed_mult;

        // ── compute steering ──────────────────────────────────────────
        let (steer_x, steer_y) = match state {
            AntState::Foraging => self.steer_foraging(i, heading, role),
            AntState::Returning => self.steer_returning(i, heading),
        };

        let (bx, by) = steering::boundary_avoidance(
            x,
            y,
            self.config.world_width,
            self.config.world_height,
            self.config.boundary_margin,
        );
        let steer_x = steer_x + bx * 3.0;
        let steer_y = steer_y + by * 3.0;

        // ── apply steering ────────────────────────────────────────────
        let desired = steer_y.atan2(steer_x);
        let diff = steering::normalize_angle(desired - heading);
        let max_turn = self.config.ant_turn_rate * dt;
        let new_heading = steering::normalize_angle(heading + diff.clamp(-max_turn, max_turn));
        self.ants.heading[i] = new_heading;

        let dx = new_heading.cos() * speed * dt;
        let dy = new_heading.sin() * speed * dt;
        let new_x = (x + dx).clamp(1.0, self.config.world_width - 1.0);
        let new_y = (y + dy).clamp(1.0, self.config.world_height - 1.0);

        self.ants.vel_x[i] = dx;
        self.ants.vel_y[i] = dy;
        self.ants.pos_x[i] = new_x;
        self.ants.pos_y[i] = new_y;

        self.ants.home_vec_x[i] += new_x - x;
        self.ants.home_vec_y[i] += new_y - y;

        // ── deposit pheromones ────────────────────────────────────────
        match state {
            AntState::Foraging => {
                self.pheromones.deposit(
                    new_x,
                    new_y,
                    PheromoneType::Home,
                    self.config.pheromone_home_deposit,
                );
            }
            AntState::Returning => {
                self.pheromones.deposit(
                    new_x,
                    new_y,
                    PheromoneType::Food,
                    self.config.pheromone_food_deposit,
                );
            }
        }
    }

    fn steer_foraging(&mut self, i: usize, heading: f32, role: u8) -> (f32, f32) {
        let x = self.ants.pos_x[i];
        let y = self.ants.pos_y[i];

        // soldiers patrol near colony instead of foraging
        if role == ROLE_SOLDIER {
            return self.steer_patrol(i, heading);
        }

        // direct vision: head toward nearest visible food
        // scouts have a wider detection radius
        let det_mult = if role == ROLE_SCOUT { 1.8 } else { 1.0 };
        let det_r_sq = (self.config.ant_detection_radius * det_mult).powi(2);
        let mut best_dist_sq = det_r_sq;
        let mut best_food: Option<(f32, f32)> = None;
        for fs in &self.food_sources {
            if fs.amount <= 0.0 {
                continue;
            }
            let dx = fs.x - x;
            let dy = fs.y - y;
            let d2 = dx * dx + dy * dy;
            if d2 < best_dist_sq {
                best_dist_sq = d2;
                best_food = Some((fs.x, fs.y));
            }
        }

        if let Some((fx, fy)) = best_food {
            return steering::seek(x, y, fx, fy);
        }

        // follow food pheromone gradient (scouts have wider sensor spread)
        let sensor_angle = if role == ROLE_SCOUT {
            self.config.ant_sensor_angle * 1.4
        } else {
            self.config.ant_sensor_angle
        };
        if let Some(angle) = self.pheromones.sense_direction(
            x,
            y,
            heading,
            self.config.ant_sensor_distance,
            sensor_angle,
            PheromoneType::Food,
        ) {
            let (px, py) = (angle.cos(), angle.sin());
            let rng_val: f32 = self.rng.gen();
            let (wx, wy) = steering::wander_direction(
                heading,
                &mut self.ants.wander_angle[i],
                self.config.ant_wander_strength * 0.4,
                rng_val,
            );
            return (px * 0.7 + wx * 0.3, py * 0.7 + wy * 0.3);
        }

        // pure wander (scouts explore more aggressively)
        let wander_str = if role == ROLE_SCOUT {
            self.config.ant_wander_strength * self.config.scout_wander_boost
        } else {
            self.config.ant_wander_strength
        };
        let rng_val: f32 = self.rng.gen();
        steering::wander_direction(
            heading,
            &mut self.ants.wander_angle[i],
            wander_str,
            rng_val,
        )
    }

    /// Soldiers orbit near their colony perimeter.
    fn steer_patrol(&mut self, i: usize, heading: f32) -> (f32, f32) {
        let x = self.ants.pos_x[i];
        let y = self.ants.pos_y[i];
        let cid = self.ants.colony_id[i] as usize;
        if cid >= self.colonies.len() {
            let r: f32 = self.rng.gen();
            return steering::wander_direction(
                heading,
                &mut self.ants.wander_angle[i],
                self.config.ant_wander_strength,
                r,
            );
        }

        let cx = self.colonies[cid].x;
        let cy = self.colonies[cid].y;
        let patrol_r = self.config.soldier_patrol_radius;

        let dx = x - cx;
        let dy = y - cy;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < patrol_r * 0.6 {
            // too close to colony, push outward
            if dist > 0.1 {
                (dx / dist, dy / dist)
            } else {
                let r: f32 = self.rng.gen();
                steering::wander_direction(
                    heading,
                    &mut self.ants.wander_angle[i],
                    self.config.ant_wander_strength,
                    r,
                )
            }
        } else if dist > patrol_r * 1.4 {
            // too far from colony, pull back
            steering::seek(x, y, cx, cy)
        } else {
            // orbit: tangent direction + slight wander
            let tangent_x = -dy / dist;
            let tangent_y = dx / dist;
            let rng_val: f32 = self.rng.gen();
            let (wx, wy) = steering::wander_direction(
                heading,
                &mut self.ants.wander_angle[i],
                self.config.ant_wander_strength * 0.5,
                rng_val,
            );
            (tangent_x * 0.7 + wx * 0.3, tangent_y * 0.7 + wy * 0.3)
        }
    }

    fn steer_returning(&mut self, i: usize, heading: f32) -> (f32, f32) {
        let x = self.ants.pos_x[i];
        let y = self.ants.pos_y[i];

        // path integration: direction toward colony
        let hx = -self.ants.home_vec_x[i];
        let hy = -self.ants.home_vec_y[i];
        let hmag = (hx * hx + hy * hy).sqrt();
        let (path_dx, path_dy) = if hmag > 0.001 {
            (hx / hmag, hy / hmag)
        } else {
            (0.0, 0.0)
        };

        // blend with home pheromone gradient
        if let Some(angle) = self.pheromones.sense_direction(
            x,
            y,
            heading,
            self.config.ant_sensor_distance,
            self.config.ant_sensor_angle,
            PheromoneType::Home,
        ) {
            let (px, py) = (angle.cos(), angle.sin());
            (path_dx * 0.4 + px * 0.6, path_dy * 0.4 + py * 0.6)
        } else {
            let rng_val: f32 = self.rng.gen();
            let (wx, wy) = steering::wander_direction(
                heading,
                &mut self.ants.wander_angle[i],
                self.config.ant_wander_strength * 0.3,
                rng_val,
            );
            (path_dx * 0.7 + wx * 0.3, path_dy * 0.7 + wy * 0.3)
        }
    }
}
