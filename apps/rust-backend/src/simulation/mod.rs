pub mod ant;
pub mod colony;
pub mod food;
pub mod pheromone;
pub mod steering;
pub mod terrain;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;

use crate::config::SimConfig;
use ant::{speed_for_role, AntState, AntStorage, ROLE_SCOUT, ROLE_SOLDIER, ROLE_WORKER};
use colony::Colony;
use food::FoodSource;
use pheromone::{PheromoneField, PheromoneType};
use terrain::Terrain;

/// Result of one ant's movement computation (produced in parallel, applied sequentially).
#[derive(Clone, Copy)]
struct AntMove {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    heading: f32,
    wander_angle: f32,
    levy_cooldown: u32,
    /// index of a food source within pickup range, -1 = none
    picked_food: i32,
    /// index of the colony the ant deposited cargo at, -1 = none
    deposited: i32,
}

pub struct SimulationState {
    pub config: SimConfig,
    pub ants: AntStorage,
    pub colonies: Vec<Colony>,
    pub food_sources: Vec<FoodSource>,
    pub pheromones: PheromoneField,
    pub terrain: Terrain,
    pub tick_count: u64,
    pub total_food_collected: f32,
    move_scratch: Vec<AntMove>,
    rng: SmallRng,
}

impl SimulationState {
    pub fn new(config: SimConfig) -> Self {
        let mut rng = SmallRng::seed_from_u64(config.terrain_seed ^ 0xA5A5_5A5A);

        let cx = config.world_width / 2.0;
        let cy = config.world_height / 2.0;

        // ── terrain ────────────────────────────────────────────────────
        let mut terrain = Terrain::generate(
            config.world_width,
            config.world_height,
            config.terrain_cell_size,
            config.terrain_seed,
            config.terrain_density,
            config.terrain_smooth_iterations,
        );
        terrain.carve_circle(cx, cy, config.colony_radius * 2.5);
        terrain.fill_unreachable(cx, cy);

        let colonies = vec![Colony {
            id: 0,
            x: cx,
            y: cy,
            radius: config.colony_radius,
            food_stored: 200.0,
            color_hue: 30,
        }];

        // ── food sources on open, reachable ground ────────────────────
        let mut food_sources = Vec::with_capacity(config.food_source_count);
        for i in 0..config.food_source_count {
            let pos = terrain.random_open_position(
                &mut rng,
                Some((cx, cy, config.food_min_distance_from_colony)),
            );
            // relax the distance constraint if the world is too cramped
            let pos = pos.or_else(|| terrain.random_open_position(&mut rng, None));
            if let Some((fx, fy)) = pos {
                food_sources.push(FoodSource {
                    id: i as u32,
                    x: fx,
                    y: fy,
                    amount: config.food_per_source,
                    max_amount: config.food_per_source,
                });
            }
        }
        // small clearings around food so it stays accessible
        for fs in &food_sources {
            terrain.carve_circle(fs.x, fs.y, 20.0);
        }

        // ── pheromones (terrain-aware) ─────────────────────────────────
        let mut pheromones = PheromoneField::new(
            config.world_width,
            config.world_height,
            config.pheromone_cell_size,
        );
        pheromones.build_blocked_mask(&terrain);

        // ── initial ants: 70% worker, 20% scout, 10% soldier ───────────
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
            terrain,
            tick_count: 0,
            total_food_collected: 0.0,
            move_scratch: Vec::new(),
            rng,
        }
    }

    pub fn tick(&mut self) {
        let dt = 1.0 / self.config.tick_rate;
        self.tick_count += 1;

        // ── phase 1: compute all ant moves in parallel (read-only) ─────
        {
            let cfg = &self.config;
            let terrain = &self.terrain;
            let pheromones = &self.pheromones;
            let food_sources = &self.food_sources;
            let colonies = &self.colonies;
            let ants = &self.ants;
            let tick = self.tick_count;
            let scratch = &mut self.move_scratch;

            (0..ants.count)
                .into_par_iter()
                .map(|i| compute_ant_move(i, dt, tick, cfg, terrain, pheromones, food_sources, colonies, ants))
                .collect_into_vec(scratch);
        }

        // ── phase 2: apply results sequentially (mutates shared state) ─
        for i in 0..self.ants.count {
            let m = self.move_scratch[i];

            if m.picked_food >= 0 {
                let j = m.picked_food as usize;
                if self.food_sources[j].amount >= 1.0 {
                    self.food_sources[j].amount -= 1.0;
                    self.ants.cargo[i] = 1.0;
                    self.ants.state[i] = AntState::Returning;
                }
            } else if m.deposited >= 0 {
                let c = m.deposited as usize;
                self.colonies[c].food_stored += self.ants.cargo[i];
                self.total_food_collected += self.ants.cargo[i];
                self.ants.cargo[i] = 0.0;
                self.ants.state[i] = AntState::Foraging;
                self.ants.home_vec_x[i] = 0.0;
                self.ants.home_vec_y[i] = 0.0;
                self.ants.energy[i] =
                    (self.ants.energy[i] + self.config.ant_energy_food_restore).min(100.0);
            }

            self.ants.home_vec_x[i] += m.x - self.ants.pos_x[i];
            self.ants.home_vec_y[i] += m.y - self.ants.pos_y[i];
            self.ants.pos_x[i] = m.x;
            self.ants.pos_y[i] = m.y;
            self.ants.vel_x[i] = m.vx;
            self.ants.vel_y[i] = m.vy;
            self.ants.heading[i] = m.heading;
            self.ants.wander_angle[i] = m.wander_angle;
            self.ants.levy_cooldown[i] = m.levy_cooldown;

            match self.ants.state[i] {
                AntState::Foraging => self.pheromones.deposit(
                    m.x,
                    m.y,
                    PheromoneType::Home,
                    self.config.pheromone_home_deposit,
                ),
                AntState::Returning => self.pheromones.deposit(
                    m.x,
                    m.y,
                    PheromoneType::Food,
                    self.config.pheromone_food_deposit,
                ),
            }
        }

        // ── phase 3: energy decay and death ────────────────────────────
        let mut i = 0;
        while i < self.ants.count {
            self.ants.energy[i] -= self.config.ant_energy_decay_per_tick;
            self.ants.age[i] += 1;
            if self.ants.energy[i] <= 0.0 {
                self.ants.remove(i);
                // swap_remove: re-process the swapped-in element at i
            } else {
                i += 1;
            }
        }

        self.spawn_ants();

        self.pheromones.evaporate(self.config.pheromone_evaporation);
        if self.tick_count % self.config.pheromone_diffusion_interval == 0 {
            self.pheromones.diffuse(self.config.pheromone_diffusion_rate);
        }
    }

    fn spawn_ants(&mut self) {
        if self.tick_count % self.config.colony_spawn_interval != 0 {
            return;
        }
        for ci in 0..self.colonies.len() {
            let mut budget = self.config.colony_spawn_batch;
            while budget > 0
                && self.ants.count < self.config.max_ants
                && self.colonies[ci].food_stored >= self.config.colony_spawn_cost
            {
                budget -= 1;
                self.colonies[ci].food_stored -= self.config.colony_spawn_cost;

                let cx = self.colonies[ci].x;
                let cy = self.colonies[ci].y;
                let cid = self.colonies[ci].id;

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
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_ant_move(
    i: usize,
    dt: f32,
    tick: u64,
    cfg: &SimConfig,
    terrain: &Terrain,
    pheromones: &PheromoneField,
    food_sources: &[FoodSource],
    colonies: &[Colony],
    ants: &AntStorage,
) -> AntMove {
    let x = ants.pos_x[i];
    let y = ants.pos_y[i];
    let heading = ants.heading[i];
    let role = ants.ant_type[i];
    let state = ants.state[i];

    // deterministic per-ant, per-tick RNG (cheap, no shared state)
    let mut rng = SmallRng::seed_from_u64(
        tick.wrapping_mul(0x517C_C1B7_2722_0A95) ^ (ants.id[i] as u64).wrapping_mul(0x2545_F491_4F6C_DD1D),
    );

    let mut m = AntMove {
        x,
        y,
        vx: 0.0,
        vy: 0.0,
        heading,
        wander_angle: ants.wander_angle[i],
        levy_cooldown: ants.levy_cooldown[i],
        picked_food: -1,
        deposited: -1,
    };

    // ── proximity events (detected here, applied sequentially) ────────
    let mut eff_state = state;
    match state {
        AntState::Foraging => {
            let pickup_r_sq = cfg.ant_pickup_radius * cfg.ant_pickup_radius;
            for (j, fs) in food_sources.iter().enumerate() {
                if fs.amount < 1.0 {
                    continue;
                }
                let dx = fs.x - x;
                let dy = fs.y - y;
                if dx * dx + dy * dy < pickup_r_sq {
                    m.picked_food = j as i32;
                    eff_state = AntState::Returning;
                    break;
                }
            }
        }
        AntState::Returning => {
            for (j, c) in colonies.iter().enumerate() {
                let dx = c.x - x;
                let dy = c.y - y;
                if dx * dx + dy * dy < c.radius * c.radius {
                    m.deposited = j as i32;
                    eff_state = AntState::Foraging;
                    m.heading = rng.gen_range(0.0..std::f32::consts::TAU);
                    m.wander_angle = 0.0;
                    break;
                }
            }
        }
    }

    // ── levy flight (foraging wanderers only) ──────────────────────────
    if eff_state == AntState::Foraging && m.levy_cooldown == 0 && rng.gen::<f32>() < 0.003 {
        m.heading = rng.gen_range(0.0..std::f32::consts::TAU);
        m.levy_cooldown = cfg.levy_cooldown_ticks;
    }
    if m.levy_cooldown > 0 {
        m.levy_cooldown -= 1;
    }
    let levy_active = m.levy_cooldown > cfg.levy_cooldown_ticks.saturating_sub(30);
    let speed_mult = if levy_active { cfg.levy_speed_boost } else { 1.0 };
    let speed = ants.speed[i] * speed_mult;

    // ── steering ───────────────────────────────────────────────────────
    let (mut steer_x, mut steer_y) = match eff_state {
        AntState::Foraging => steer_foraging(
            i, x, y, m.heading, role, cfg, pheromones, food_sources, colonies, ants,
            &mut m.wander_angle, &mut rng,
        ),
        AntState::Returning => steer_returning(
            i, x, y, m.heading, cfg, pheromones, ants, &mut m.wander_angle, &mut rng,
        ),
    };

    let (bx, by) = steering::boundary_avoidance(
        x,
        y,
        cfg.world_width,
        cfg.world_height,
        cfg.boundary_margin,
    );
    steer_x += bx * 3.0;
    steer_y += by * 3.0;

    if let Some((wx, wy)) = steering::wall_avoidance(terrain, x, y, m.heading, cfg.wall_probe_distance) {
        steer_x += wx * 4.0;
        steer_y += wy * 4.0;
    }

    // ── integrate ──────────────────────────────────────────────────────
    let desired = steer_y.atan2(steer_x);
    let diff = steering::normalize_angle(desired - m.heading);
    let max_turn = cfg.ant_turn_rate * dt;
    let new_heading = steering::normalize_angle(m.heading + diff.clamp(-max_turn, max_turn));
    m.heading = new_heading;

    let dx = new_heading.cos() * speed * dt;
    let dy = new_heading.sin() * speed * dt;
    let mut nx = (x + dx).clamp(1.0, cfg.world_width - 1.0);
    let mut ny = (y + dy).clamp(1.0, cfg.world_height - 1.0);

    // ── terrain collision: slide along walls, turn around in dead ends ─
    if terrain.is_solid_at(nx, ny) {
        if !terrain.is_solid_at(nx, y) {
            ny = y;
        } else if !terrain.is_solid_at(x, ny) {
            nx = x;
        } else {
            nx = x;
            ny = y;
            m.heading = steering::normalize_angle(new_heading + std::f32::consts::PI);
        }
    }

    m.vx = nx - x;
    m.vy = ny - y;
    m.x = nx;
    m.y = ny;
    m
}

#[allow(clippy::too_many_arguments)]
fn steer_foraging(
    i: usize,
    x: f32,
    y: f32,
    heading: f32,
    role: u8,
    cfg: &SimConfig,
    pheromones: &PheromoneField,
    food_sources: &[FoodSource],
    colonies: &[Colony],
    ants: &AntStorage,
    wander_angle: &mut f32,
    rng: &mut SmallRng,
) -> (f32, f32) {
    // soldiers patrol near their colony instead of foraging
    if role == ROLE_SOLDIER {
        return steer_patrol(x, y, heading, ants.colony_id[i], colonies, cfg, wander_angle, rng);
    }

    // direct vision: head toward the nearest visible food (scouts see further)
    let det_mult = if role == ROLE_SCOUT { 1.8 } else { 1.0 };
    let det_r_sq = (cfg.ant_detection_radius * det_mult).powi(2);
    let mut best_dist_sq = det_r_sq;
    let mut best_food: Option<(f32, f32)> = None;
    for fs in food_sources {
        if fs.amount < 1.0 {
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

    // follow the food pheromone gradient (scouts use a wider sensor spread)
    let sensor_angle = if role == ROLE_SCOUT {
        cfg.ant_sensor_angle * 1.4
    } else {
        cfg.ant_sensor_angle
    };
    if let Some(angle) = pheromones.sense_direction(
        x,
        y,
        heading,
        cfg.ant_sensor_distance,
        sensor_angle,
        PheromoneType::Food,
    ) {
        let (px, py) = (angle.cos(), angle.sin());
        let rng_val: f32 = rng.gen();
        let (wx, wy) =
            steering::wander_direction(heading, wander_angle, cfg.ant_wander_strength * 0.4, rng_val);
        return (px * 0.7 + wx * 0.3, py * 0.7 + wy * 0.3);
    }

    // pure wander (scouts explore more aggressively)
    let wander_str = if role == ROLE_SCOUT {
        cfg.ant_wander_strength * cfg.scout_wander_boost
    } else {
        cfg.ant_wander_strength
    };
    let rng_val: f32 = rng.gen();
    steering::wander_direction(heading, wander_angle, wander_str, rng_val)
}

#[allow(clippy::too_many_arguments)]
fn steer_patrol(
    x: f32,
    y: f32,
    heading: f32,
    colony_id: u32,
    colonies: &[Colony],
    cfg: &SimConfig,
    wander_angle: &mut f32,
    rng: &mut SmallRng,
) -> (f32, f32) {
    let colony = colonies.iter().find(|c| c.id == colony_id);
    let Some(colony) = colony else {
        let r: f32 = rng.gen();
        return steering::wander_direction(heading, wander_angle, cfg.ant_wander_strength, r);
    };

    let patrol_r = cfg.soldier_patrol_radius;
    let dx = x - colony.x;
    let dy = y - colony.y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < patrol_r * 0.6 {
        if dist > 0.1 {
            (dx / dist, dy / dist)
        } else {
            let r: f32 = rng.gen();
            steering::wander_direction(heading, wander_angle, cfg.ant_wander_strength, r)
        }
    } else if dist > patrol_r * 1.4 {
        steering::seek(x, y, colony.x, colony.y)
    } else {
        // orbit: tangent direction + slight wander
        let tangent_x = -dy / dist;
        let tangent_y = dx / dist;
        let rng_val: f32 = rng.gen();
        let (wx, wy) =
            steering::wander_direction(heading, wander_angle, cfg.ant_wander_strength * 0.5, rng_val);
        (tangent_x * 0.7 + wx * 0.3, tangent_y * 0.7 + wy * 0.3)
    }
}

#[allow(clippy::too_many_arguments)]
fn steer_returning(
    i: usize,
    x: f32,
    y: f32,
    heading: f32,
    cfg: &SimConfig,
    pheromones: &PheromoneField,
    ants: &AntStorage,
    wander_angle: &mut f32,
    rng: &mut SmallRng,
) -> (f32, f32) {
    // path integration: direction toward the colony
    let hx = -ants.home_vec_x[i];
    let hy = -ants.home_vec_y[i];
    let hmag = (hx * hx + hy * hy).sqrt();
    let (path_dx, path_dy) = if hmag > 0.001 {
        (hx / hmag, hy / hmag)
    } else {
        (0.0, 0.0)
    };

    // blend with the home pheromone gradient
    if let Some(angle) = pheromones.sense_direction(
        x,
        y,
        heading,
        cfg.ant_sensor_distance,
        cfg.ant_sensor_angle,
        PheromoneType::Home,
    ) {
        let (px, py) = (angle.cos(), angle.sin());
        (path_dx * 0.4 + px * 0.6, path_dy * 0.4 + py * 0.6)
    } else {
        let rng_val: f32 = rng.gen();
        let (wx, wy) =
            steering::wander_direction(heading, wander_angle, cfg.ant_wander_strength * 0.3, rng_val);
        (path_dx * 0.7 + wx * 0.3, path_dy * 0.7 + wy * 0.3)
    }
}
