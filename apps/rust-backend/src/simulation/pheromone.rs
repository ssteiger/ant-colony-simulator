use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PheromoneType {
    Food,
    Home,
}

/// Grid-based pheromone field with separate layers per type.
/// Much faster than storing individual trail entities.
#[derive(Serialize, Deserialize)]
pub struct PheromoneField {
    pub grid_w: usize,
    pub grid_h: usize,
    pub cell_size: f32,
    pub food: Vec<f32>,
    pub home: Vec<f32>,
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
        }
    }

    fn layer(&self, ptype: PheromoneType) -> &[f32] {
        match ptype {
            PheromoneType::Food => &self.food,
            PheromoneType::Home => &self.home,
        }
    }

    fn layer_mut(&mut self, ptype: PheromoneType) -> &mut [f32] {
        match ptype {
            PheromoneType::Food => &mut self.food,
            PheromoneType::Home => &mut self.home,
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
            let layer = self.layer_mut(ptype);
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

    pub fn evaporate(&mut self, factor: f32) {
        for v in &mut self.food {
            *v *= factor;
        }
        for v in &mut self.home {
            *v *= factor;
        }
    }

    pub fn diffuse(&mut self, rate: f32) {
        diffuse_layer(&mut self.food, self.grid_w, self.grid_h, rate);
        diffuse_layer(&mut self.home, self.grid_w, self.grid_h, rate);
    }
}

fn diffuse_layer(layer: &mut [f32], w: usize, h: usize, rate: f32) {
    let src: Vec<f32> = layer.to_vec();
    let keep = 1.0 - rate;

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
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
                        sum += src[ny as usize * w + nx as usize];
                        count += 1;
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
