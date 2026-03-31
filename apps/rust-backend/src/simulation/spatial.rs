use std::collections::HashMap;

/// Uniform-grid spatial index rebuilt every tick for O(1) neighbor lookups.
pub struct SpatialGrid {
    cell_size: f32,
    inv_cell_size: f32,
    cells: HashMap<(i32, i32), Vec<usize>>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            inv_cell_size: 1.0 / cell_size,
            cells: HashMap::with_capacity(256),
        }
    }

    fn cell_coord(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x * self.inv_cell_size).floor() as i32,
            (y * self.inv_cell_size).floor() as i32,
        )
    }

    pub fn rebuild(&mut self, pos_x: &[f32], pos_y: &[f32], count: usize) {
        for bucket in self.cells.values_mut() {
            bucket.clear();
        }
        for i in 0..count {
            let coord = self.cell_coord(pos_x[i], pos_y[i]);
            self.cells.entry(coord).or_default().push(i);
        }
    }

    /// Return indices of all entities within `radius` of (x, y).
    /// Results are approximate -- callers should do a final distance check.
    pub fn query_radius(&self, x: f32, y: f32, radius: f32, results: &mut Vec<usize>) {
        results.clear();
        let r_cells = (radius * self.inv_cell_size).ceil() as i32;
        let (cx, cy) = self.cell_coord(x, y);

        for dy in -r_cells..=r_cells {
            for dx in -r_cells..=r_cells {
                if let Some(indices) = self.cells.get(&(cx + dx, cy + dy)) {
                    results.extend_from_slice(indices);
                }
            }
        }
    }
}
