use crate::server::messages::*;
use super::ant::AntState;
use super::SimulationState;

impl SimulationState {
    pub fn full_snapshot(&self, simulation_id: u32) -> ServerMessage {
        ServerMessage::FullState {
            simulation_id,
            tick: self.tick_count,
            world_width: self.config.world_width,
            world_height: self.config.world_height,
            ants: self.snapshot_ants(),
            colonies: self.snapshot_colonies(),
            food_sources: self.snapshot_food(),
            pheromone_grid: self.snapshot_pheromones(),
        }
    }

    pub fn delta_snapshot(&self, simulation_id: u32) -> ServerMessage {
        ServerMessage::DeltaUpdate {
            simulation_id,
            tick: self.tick_count,
            updated_ants: self.snapshot_ants(),
            updated_colonies: self.snapshot_colonies(),
            updated_food_sources: self.snapshot_food(),
            removed_ant_ids: Vec::new(),
            removed_food_source_ids: Vec::new(),
        }
    }

    fn snapshot_ants(&self) -> Vec<AntSnapshot> {
        (0..self.ants.count)
            .map(|i| AntSnapshot {
                id: self.ants.id[i],
                position: [self.ants.pos_x[i], self.ants.pos_y[i]],
                angle: self.ants.heading[i],
                colony_id: self.ants.colony_id[i],
                ant_type_id: self.ants.ant_type[i],
                state: match self.ants.state[i] {
                    AntState::Foraging => "foraging".into(),
                    AntState::Returning => "returning".into(),
                },
                speed: self.ants.speed[i],
                health: self.ants.health[i],
                energy: self.ants.energy[i],
            })
            .collect()
    }

    fn snapshot_colonies(&self) -> Vec<ColonySnapshot> {
        self.colonies
            .iter()
            .map(|c| {
                let pop = self
                    .ants
                    .colony_id
                    .iter()
                    .filter(|&&cid| cid == c.id)
                    .count();
                ColonySnapshot {
                    id: c.id,
                    center: [c.x, c.y],
                    radius: c.radius,
                    population: pop,
                    food_stored: c.food_stored,
                    color_hue: c.color_hue,
                }
            })
            .collect()
    }

    fn snapshot_food(&self) -> Vec<FoodSnapshot> {
        self.food_sources
            .iter()
            .filter(|f| f.amount > 0.0)
            .map(|f| FoodSnapshot {
                id: f.id,
                position: [f.x, f.y],
                food_type: "generic".into(),
                amount: f.amount,
            })
            .collect()
    }

    /// Encode pheromone grids as u8 (0-255) for compact transfer.
    fn snapshot_pheromones(&self) -> PheromoneSnapshot {
        let to_u8 = |v: &[f32]| -> Vec<u8> {
            v.iter().map(|&x| (x.clamp(0.0, 1.0) * 255.0) as u8).collect()
        };
        PheromoneSnapshot {
            grid_w: self.pheromones.grid_w,
            grid_h: self.pheromones.grid_h,
            cell_size: self.pheromones.cell_size,
            food: to_u8(&self.pheromones.food),
            home: to_u8(&self.pheromones.home),
        }
    }
}
