//! Binary WebSocket frame encoding (all values little-endian).
//!
//! Frame layouts:
//!
//! INIT (1):      u8 type, u8 version, i32 sim_id, u64 tick,
//!                f32 world_w, f32 world_h,
//!                u32 terrain_w, u32 terrain_h, f32 terrain_cell,
//!                u32 pher_w, u32 pher_h, f32 pher_cell,
//!                u16 colony_count x { u32 id, f32 x, f32 y, f32 radius, u16 hue },
//!                u16 food_count x { u32 id, f32 x, f32 y, f32 amount, f32 max },
//!                terrain bits (ceil(w*h/8) bytes, LSB-first)
//!
//! ANTS (2):      u8 type, u64 tick, u32 count x { u16 qx, u16 qy, u8 heading, u8 flags }
//!                qx/qy quantized to 0..65535 over world size,
//!                heading quantized to 0..255 over 2*PI,
//!                flags: bits 0-1 role, bit 2 carrying/returning
//!
//! PHEROMONE (3): u8 type, u64 tick, u32 w, u32 h, w*h u8 food, w*h u8 home
//!
//! FOOD (4):      u8 type, u64 tick, u16 count x { u32 id, f32 amount }

use crate::simulation::ant::AntState;
use crate::simulation::SimulationState;

pub const FRAME_INIT: u8 = 1;
pub const FRAME_ANTS: u8 = 2;
pub const FRAME_PHEROMONE: u8 = 3;
pub const FRAME_FOOD: u8 = 4;
pub const PROTOCOL_VERSION: u8 = 1;

struct Writer(Vec<u8>);

impl Writer {
    fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }
    #[inline]
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    #[inline]
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    #[inline]
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    #[inline]
    fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    #[inline]
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    #[inline]
    fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn bytes(&mut self, v: &[u8]) {
        self.0.extend_from_slice(v);
    }
}

pub fn encode_init(sim: &SimulationState) -> Vec<u8> {
    let terrain_bits = sim.terrain.packed_bits();
    let mut w = Writer::with_capacity(64 + terrain_bits.len() + sim.food_sources.len() * 20);

    w.u8(FRAME_INIT);
    w.u8(PROTOCOL_VERSION);
    w.i32(sim.config.simulation_id);
    w.u64(sim.tick_count);
    w.f32(sim.config.world_width);
    w.f32(sim.config.world_height);

    w.u32(sim.terrain.grid_w as u32);
    w.u32(sim.terrain.grid_h as u32);
    w.f32(sim.terrain.cell_size);

    w.u32(sim.pheromones.grid_w as u32);
    w.u32(sim.pheromones.grid_h as u32);
    w.f32(sim.pheromones.cell_size);

    w.u16(sim.colonies.len() as u16);
    for c in &sim.colonies {
        w.u32(c.id);
        w.f32(c.x);
        w.f32(c.y);
        w.f32(c.radius);
        w.u16(c.color_hue);
    }

    w.u16(sim.food_sources.len() as u16);
    for f in &sim.food_sources {
        w.u32(f.id);
        w.f32(f.x);
        w.f32(f.y);
        w.f32(f.amount);
        w.f32(f.max_amount);
    }

    w.bytes(&terrain_bits);
    w.0
}

pub fn encode_ants(sim: &SimulationState) -> Vec<u8> {
    let count = sim.ants.count;
    let mut w = Writer::with_capacity(16 + count * 6);

    w.u8(FRAME_ANTS);
    w.u64(sim.tick_count);
    w.u32(count as u32);

    let sx = 65535.0 / sim.config.world_width;
    let sy = 65535.0 / sim.config.world_height;
    let sh = 256.0 / std::f32::consts::TAU;

    for i in 0..count {
        let qx = (sim.ants.pos_x[i] * sx).clamp(0.0, 65535.0) as u16;
        let qy = (sim.ants.pos_y[i] * sy).clamp(0.0, 65535.0) as u16;
        // heading is normalized to [-PI, PI]; shift into [0, TAU)
        let mut h = sim.ants.heading[i];
        if h < 0.0 {
            h += std::f32::consts::TAU;
        }
        let qh = (h * sh) as i32 & 0xFF;
        let carrying = (sim.ants.state[i] == AntState::Returning) as u8;
        let flags = (sim.ants.ant_type[i] & 0b11) | (carrying << 2);

        w.u16(qx);
        w.u16(qy);
        w.u8(qh as u8);
        w.u8(flags);
    }
    w.0
}

pub fn encode_pheromones(sim: &SimulationState) -> Vec<u8> {
    let pw = sim.pheromones.grid_w;
    let ph = sim.pheromones.grid_h;
    let mut w = Writer::with_capacity(24 + pw * ph * 2);

    w.u8(FRAME_PHEROMONE);
    w.u64(sim.tick_count);
    w.u32(pw as u32);
    w.u32(ph as u32);

    for &v in &sim.pheromones.food {
        w.u8((v.clamp(0.0, 1.0) * 255.0) as u8);
    }
    for &v in &sim.pheromones.home {
        w.u8((v.clamp(0.0, 1.0) * 255.0) as u8);
    }
    w.0
}

pub fn encode_food(sim: &SimulationState) -> Vec<u8> {
    let mut w = Writer::with_capacity(16 + sim.food_sources.len() * 8);
    w.u8(FRAME_FOOD);
    w.u64(sim.tick_count);
    w.u16(sim.food_sources.len() as u16);
    for f in &sim.food_sources {
        w.u32(f.id);
        w.f32(f.amount);
    }
    w.0
}

/// Human-readable stats for the HUD, sent as JSON text at ~1 Hz.
pub fn encode_stats_json(sim: &SimulationState, tps: f32) -> String {
    let colony_food: f32 = sim.colonies.iter().map(|c| c.food_stored).sum();
    let world_food: f32 = sim.food_sources.iter().map(|f| f.amount).sum();
    serde_json::json!({
        "type": "stats",
        "simulationId": sim.config.simulation_id,
        "tick": sim.tick_count,
        "antCount": sim.ants.count,
        "foodCollected": sim.total_food_collected,
        "colonyFood": colony_food,
        "worldFood": world_food,
        "tps": tps,
    })
    .to_string()
}
