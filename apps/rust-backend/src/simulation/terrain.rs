use std::collections::VecDeque;

use rand::rngs::SmallRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Solid/open occupancy grid generated from seeded fBm value-noise,
/// smoothed with a cellular-automata pass for cave-like shapes.
#[derive(Clone, Serialize, Deserialize)]
pub struct Terrain {
    pub grid_w: usize,
    pub grid_h: usize,
    pub cell_size: f32,
    /// 1 = solid rock, 0 = open ground. Row-major, indexed `gy * grid_w + gx`.
    pub solid: Vec<u8>,
}

fn hash01(ix: i64, iy: i64, seed: u64) -> f32 {
    let mut h = seed
        ^ (ix as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (iy as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    (h & 0x00FF_FFFF) as f32 / 16_777_216.0
}

fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let tx = x - x0;
    let ty = y - y0;
    let sx = tx * tx * (3.0 - 2.0 * tx);
    let sy = ty * ty * (3.0 - 2.0 * ty);
    let ix = x0 as i64;
    let iy = y0 as i64;
    let v00 = hash01(ix, iy, seed);
    let v10 = hash01(ix + 1, iy, seed);
    let v01 = hash01(ix, iy + 1, seed);
    let v11 = hash01(ix + 1, iy + 1, seed);
    let a = v00 + (v10 - v00) * sx;
    let b = v01 + (v11 - v01) * sx;
    a + (b - a) * sy
}

fn fbm(x: f32, y: f32, seed: u64, octaves: u32) -> f32 {
    let mut sum = 0.0f32;
    let mut amp = 1.0f32;
    let mut freq = 1.0f32;
    let mut norm = 0.0f32;
    for o in 0..octaves {
        sum += amp * value_noise(x * freq, y * freq, seed.wrapping_add(o as u64 * 1013));
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}

impl Terrain {
    /// Generate terrain. `density` is the target fraction of solid cells (0..~0.5).
    pub fn generate(
        world_w: f32,
        world_h: f32,
        cell_size: f32,
        seed: u64,
        density: f32,
        smooth_iterations: u32,
    ) -> Self {
        let grid_w = (world_w / cell_size).ceil() as usize;
        let grid_h = (world_h / cell_size).ceil() as usize;
        let n = grid_w * grid_h;

        // feature size of roughly 16 cells (~128 world units at cell_size 8)
        let noise_scale = 0.06f32;
        let mut values = vec![0.0f32; n];
        for gy in 0..grid_h {
            for gx in 0..grid_w {
                values[gy * grid_w + gx] =
                    fbm(gx as f32 * noise_scale, gy as f32 * noise_scale, seed, 4);
            }
        }

        // threshold at the quantile that yields the requested density
        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let q = ((1.0 - density.clamp(0.0, 0.95)) * (n - 1) as f32) as usize;
        let threshold = sorted[q];

        let mut solid: Vec<u8> = values
            .iter()
            .map(|&v| if v > threshold { 1 } else { 0 })
            .collect();

        let mut terrain = Self {
            grid_w,
            grid_h,
            cell_size,
            solid: Vec::new(),
        };

        // cellular-automata smoothing for organic cave shapes
        for _ in 0..smooth_iterations {
            let src = solid.clone();
            for gy in 0..grid_h {
                for gx in 0..grid_w {
                    let mut walls = 0u32;
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = gx as i32 + dx;
                            let ny = gy as i32 + dy;
                            if nx < 0
                                || ny < 0
                                || nx as usize >= grid_w
                                || ny as usize >= grid_h
                                || src[ny as usize * grid_w + nx as usize] == 1
                            {
                                walls += 1;
                            }
                        }
                    }
                    let idx = gy * grid_w + gx;
                    solid[idx] = if walls >= 5 {
                        1
                    } else if walls <= 2 {
                        0
                    } else {
                        src[idx]
                    };
                }
            }
        }

        // solid border so ants can never leave the world
        for gx in 0..grid_w {
            solid[gx] = 1;
            solid[(grid_h - 1) * grid_w + gx] = 1;
        }
        for gy in 0..grid_h {
            solid[gy * grid_w] = 1;
            solid[gy * grid_w + grid_w - 1] = 1;
        }

        terrain.solid = solid;
        terrain
    }

    #[inline]
    pub fn idx(&self, gx: usize, gy: usize) -> usize {
        gy * self.grid_w + gx
    }

    /// Out-of-bounds counts as solid.
    #[inline]
    pub fn is_solid_cell(&self, gx: i32, gy: i32) -> bool {
        if gx < 0 || gy < 0 || gx as usize >= self.grid_w || gy as usize >= self.grid_h {
            return true;
        }
        self.solid[gy as usize * self.grid_w + gx as usize] == 1
    }

    #[inline]
    pub fn is_solid_at(&self, x: f32, y: f32) -> bool {
        let gx = (x / self.cell_size).floor() as i32;
        let gy = (y / self.cell_size).floor() as i32;
        self.is_solid_cell(gx, gy)
    }

    /// Open up a circular clearing centered at world position (x, y).
    /// Never carves the 1-cell world border.
    pub fn carve_circle(&mut self, x: f32, y: f32, radius: f32) {
        let r_cells = (radius / self.cell_size).ceil() as i32;
        let cx = (x / self.cell_size).floor() as i32;
        let cy = (y / self.cell_size).floor() as i32;
        let r2 = (radius / self.cell_size).powi(2);
        for dy in -r_cells..=r_cells {
            for dx in -r_cells..=r_cells {
                if (dx * dx + dy * dy) as f32 > r2 {
                    continue;
                }
                let gx = cx + dx;
                let gy = cy + dy;
                if gx <= 0
                    || gy <= 0
                    || gx as usize >= self.grid_w - 1
                    || gy as usize >= self.grid_h - 1
                {
                    continue;
                }
                let idx = self.idx(gx as usize, gy as usize);
                self.solid[idx] = 0;
            }
        }
    }

    /// Flood-fill (4-connectivity) from the given world position and turn every
    /// unreachable open cell into solid, guaranteeing full connectivity.
    pub fn fill_unreachable(&mut self, from_x: f32, from_y: f32) {
        let start_x = (from_x / self.cell_size).floor() as i32;
        let start_y = (from_y / self.cell_size).floor() as i32;
        if self.is_solid_cell(start_x, start_y) {
            return;
        }

        let mut visited = vec![false; self.grid_w * self.grid_h];
        let mut queue = VecDeque::new();
        let start = self.idx(start_x as usize, start_y as usize);
        visited[start] = true;
        queue.push_back((start_x, start_y));

        while let Some((gx, gy)) = queue.pop_front() {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nx = gx + dx;
                let ny = gy + dy;
                if self.is_solid_cell(nx, ny) {
                    continue;
                }
                let idx = self.idx(nx as usize, ny as usize);
                if !visited[idx] {
                    visited[idx] = true;
                    queue.push_back((nx, ny));
                }
            }
        }

        for i in 0..self.solid.len() {
            if self.solid[i] == 0 && !visited[i] {
                self.solid[i] = 1;
            }
        }
    }

    /// Random open world position, optionally at a minimum distance from a point.
    pub fn random_open_position(
        &self,
        rng: &mut SmallRng,
        min_dist_from: Option<(f32, f32, f32)>,
    ) -> Option<(f32, f32)> {
        for _ in 0..20_000 {
            let gx = rng.gen_range(1..self.grid_w - 1);
            let gy = rng.gen_range(1..self.grid_h - 1);
            if self.solid[self.idx(gx, gy)] == 1 {
                continue;
            }
            let x = (gx as f32 + 0.5) * self.cell_size;
            let y = (gy as f32 + 0.5) * self.cell_size;
            if let Some((px, py, min_d)) = min_dist_from {
                let dx = x - px;
                let dy = y - py;
                if dx * dx + dy * dy < min_d * min_d {
                    continue;
                }
            }
            return Some((x, y));
        }
        None
    }

    /// Bit-packed grid (LSB-first within each byte) for the wire protocol.
    pub fn packed_bits(&self) -> Vec<u8> {
        let n = self.solid.len();
        let mut out = vec![0u8; n.div_ceil(8)];
        for (i, &s) in self.solid.iter().enumerate() {
            if s == 1 {
                out[i / 8] |= 1 << (i % 8);
            }
        }
        out
    }
}
