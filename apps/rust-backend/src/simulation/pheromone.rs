use serde::{Deserialize, Serialize};

use super::terrain::Terrain;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PheromoneType {
    Food,
    Home,
}

/// Grid-based pheromone field with separate layers per type.
/// Much faster than storing individual trail entities.
#[derive(Clone, Serialize, Deserialize)]
pub struct PheromoneField {
    pub grid_w: usize,
    pub grid_h: usize,
    pub cell_size: f32,
    pub food: Vec<f32>,
    pub home: Vec<f32>,
    /// 1 = cell lies inside solid terrain; no deposit or diffusion there.
    pub blocked: Vec<u8>,
}

impl PheromoneField {
    pub fn new(world_width: f32, world_height: f32, cell_size: f32) -> Self {
        let grid_w = (world_width / cell_size).ceil() as usize;
        let grid_h = (world_height / cell_size).ceil() as usize;
        let size = grid_w * grid_h;
        Self {
            grid_w,
            grid_h,
            cell_size,
            food: vec![0.0; size],
            home: vec![0.0; size],
            blocked: vec![0; size],
        }
    }

    /// Mark pheromone cells whose center lies inside solid terrain as blocked.
    pub fn build_blocked_mask(&mut self, terrain: &Terrain) {
        for gy in 0..self.grid_h {
            for gx in 0..self.grid_w {
                let x = (gx as f32 + 0.5) * self.cell_size;
                let y = (gy as f32 + 0.5) * self.cell_size;
                self.blocked[gy * self.grid_w + gx] = terrain.is_solid_at(x, y) as u8;
            }
        }
    }

    /// Seed a static radial "home" gradient anchored to the colonies. The field
    /// is strongest at a nest and falls off linearly with distance, so a
    /// returning ant can always climb the gradient back toward the nearest nest.
    ///
    /// Unlike the food layer, this is navigation infrastructure: it is computed
    /// once and intentionally excluded from evaporation and diffusion. This
    /// replaces the previous behavior where wandering ants emitted home
    /// pheromone, which made the "home" field a population-density blob rather
    /// than real directional information.
    pub fn seed_home_field(&mut self, colonies: &[(f32, f32)]) {
        if colonies.is_empty() {
            return;
        }
        let world_w = self.grid_w as f32 * self.cell_size;
        let world_h = self.grid_h as f32 * self.cell_size;
        // Range spans the world diagonal so every reachable cell keeps a
        // positive, detectable gradient toward the nest.
        let range = (world_w * world_w + world_h * world_h).sqrt();

        for gy in 0..self.grid_h {
            for gx in 0..self.grid_w {
                let idx = self.idx(gx, gy);
                if self.blocked[idx] == 1 {
                    self.home[idx] = 0.0;
                    continue;
                }
                let cx = (gx as f32 + 0.5) * self.cell_size;
                let cy = (gy as f32 + 0.5) * self.cell_size;
                let mut best = 0.0f32;
                for &(colx, coly) in colonies {
                    let dx = cx - colx;
                    let dy = cy - coly;
                    let d = (dx * dx + dy * dy).sqrt();
                    let v = (1.0 - d / range).max(0.0);
                    if v > best {
                        best = v;
                    }
                }
                self.home[idx] = best;
            }
        }
    }

    fn layer(&self, ptype: PheromoneType) -> &[f32] {
        match ptype {
            PheromoneType::Food => &self.food,
            PheromoneType::Home => &self.home,
        }
    }

    fn to_grid(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        let gx = (x / self.cell_size) as i32;
        let gy = (y / self.cell_size) as i32;
        if gx >= 0 && gy >= 0 && (gx as usize) < self.grid_w && (gy as usize) < self.grid_h {
            Some((gx as usize, gy as usize))
        } else {
            None
        }
    }

    #[inline]
    fn idx(&self, gx: usize, gy: usize) -> usize {
        gy * self.grid_w + gx
    }

    pub fn deposit(&mut self, x: f32, y: f32, ptype: PheromoneType, amount: f32) {
        if let Some((gx, gy)) = self.to_grid(x, y) {
            let idx = self.idx(gx, gy);
            if self.blocked[idx] == 1 {
                return;
            }
            let layer = match ptype {
                PheromoneType::Food => &mut self.food,
                PheromoneType::Home => &mut self.home,
            };
            layer[idx] = (layer[idx] + amount).min(1.0);
        }
    }

    pub fn sample(&self, x: f32, y: f32, ptype: PheromoneType) -> f32 {
        if let Some((gx, gy)) = self.to_grid(x, y) {
            self.layer(ptype)[self.idx(gx, gy)]
        } else {
            0.0
        }
    }

    /// Three-sensor biological model: sample left, center, right ahead of the ant.
    /// Returns the angle toward the strongest pheromone signal, or None if nothing detected.
    pub fn sense_direction(
        &self,
        x: f32,
        y: f32,
        heading: f32,
        sensor_dist: f32,
        sensor_angle: f32,
        ptype: PheromoneType,
    ) -> Option<f32> {
        let left = heading - sensor_angle;
        let center = heading;
        let right = heading + sensor_angle;

        let sl = self.sample(
            x + left.cos() * sensor_dist,
            y + left.sin() * sensor_dist,
            ptype,
        );
        let sc = self.sample(
            x + center.cos() * sensor_dist,
            y + center.sin() * sensor_dist,
            ptype,
        );
        let sr = self.sample(
            x + right.cos() * sensor_dist,
            y + right.sin() * sensor_dist,
            ptype,
        );

        let max_val = sl.max(sc).max(sr);
        if max_val < 0.001 {
            return None;
        }

        if sc >= sl && sc >= sr {
            Some(center)
        } else if sl > sr {
            Some(left)
        } else {
            Some(right)
        }
    }

    /// Evaporate the food (recruitment) layer. The home layer is a static
    /// gradient seeded by `seed_home_field` and is intentionally left untouched.
    pub fn evaporate(&mut self, factor: f32) {
        for v in &mut self.food {
            *v *= factor;
        }
    }

    /// Diffuse the food layer only; the home layer is static infrastructure.
    pub fn diffuse(&mut self, rate: f32) {
        diffuse_layer(&mut self.food, &self.blocked, self.grid_w, self.grid_h, rate);
    }
}

fn diffuse_layer(layer: &mut [f32], blocked: &[u8], w: usize, h: usize, rate: f32) {
    let src: Vec<f32> = layer.to_vec();
    let keep = 1.0 - rate;

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            if blocked[idx] == 1 {
                continue;
            }
            let center = src[idx];
            if center < 0.0001 {
                continue;
            }

            let mut sum = 0.0f32;
            let mut count = 0u32;

            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                        let nidx = ny as usize * w + nx as usize;
                        if blocked[nidx] == 0 {
                            sum += src[nidx];
                            count += 1;
                        }
                    }
                }
            }

            let avg_neighbor = if count > 0 {
                sum / count as f32
            } else {
                0.0
            };
            layer[idx] = center * keep + avg_neighbor * rate;
        }
    }
}
