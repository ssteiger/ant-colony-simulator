use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum AntState {
    Foraging,
    Returning,
}

/// 0=worker, 1=scout, 2=soldier
pub const ROLE_WORKER: u8 = 0;
pub const ROLE_SCOUT: u8 = 1;
pub const ROLE_SOLDIER: u8 = 2;

/// Structure-of-Arrays storage for all ant data.
/// Each field is a parallel Vec indexed by the ant's slot.
#[derive(Serialize, Deserialize)]
pub struct AntStorage {
    pub count: usize,
    next_id: u32,

    pub id: Vec<u32>,
    pub pos_x: Vec<f32>,
    pub pos_y: Vec<f32>,
    pub vel_x: Vec<f32>,
    pub vel_y: Vec<f32>,
    pub heading: Vec<f32>,
    pub speed: Vec<f32>,

    pub state: Vec<AntState>,
    pub colony_id: Vec<u32>,
    pub ant_type: Vec<u8>,

    pub cargo: Vec<f32>,
    pub energy: Vec<f32>,
    pub health: Vec<f32>,
    pub age: Vec<u64>,

    pub home_vec_x: Vec<f32>,
    pub home_vec_y: Vec<f32>,
    pub wander_angle: Vec<f32>,
    pub levy_cooldown: Vec<u32>,
}

impl AntStorage {
    pub fn new() -> Self {
        Self {
            count: 0,
            next_id: 0,
            id: Vec::new(),
            pos_x: Vec::new(),
            pos_y: Vec::new(),
            vel_x: Vec::new(),
            vel_y: Vec::new(),
            heading: Vec::new(),
            speed: Vec::new(),
            state: Vec::new(),
            colony_id: Vec::new(),
            ant_type: Vec::new(),
            cargo: Vec::new(),
            energy: Vec::new(),
            health: Vec::new(),
            age: Vec::new(),
            home_vec_x: Vec::new(),
            home_vec_y: Vec::new(),
            wander_angle: Vec::new(),
            levy_cooldown: Vec::new(),
        }
    }

    pub fn add(
        &mut self,
        x: f32,
        y: f32,
        colony_id: u32,
        ant_type: u8,
        speed: f32,
        heading: f32,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.count += 1;

        self.id.push(id);
        self.pos_x.push(x);
        self.pos_y.push(y);
        self.vel_x.push(0.0);
        self.vel_y.push(0.0);
        self.heading.push(heading);
        self.speed.push(speed);
        self.state.push(AntState::Foraging);
        self.colony_id.push(colony_id);
        self.ant_type.push(ant_type);
        self.cargo.push(0.0);
        self.energy.push(100.0);
        self.health.push(100.0);
        self.age.push(0);
        self.home_vec_x.push(0.0);
        self.home_vec_y.push(0.0);
        self.wander_angle.push(0.0);
        self.levy_cooldown.push(0);

        id
    }

    /// Swap-remove ant at index `i`. Caller must handle index invalidation.
    pub fn remove(&mut self, i: usize) {
        self.count -= 1;
        self.id.swap_remove(i);
        self.pos_x.swap_remove(i);
        self.pos_y.swap_remove(i);
        self.vel_x.swap_remove(i);
        self.vel_y.swap_remove(i);
        self.heading.swap_remove(i);
        self.speed.swap_remove(i);
        self.state.swap_remove(i);
        self.colony_id.swap_remove(i);
        self.ant_type.swap_remove(i);
        self.cargo.swap_remove(i);
        self.energy.swap_remove(i);
        self.health.swap_remove(i);
        self.age.swap_remove(i);
        self.home_vec_x.swap_remove(i);
        self.home_vec_y.swap_remove(i);
        self.wander_angle.swap_remove(i);
        self.levy_cooldown.swap_remove(i);
    }
}

pub fn speed_for_role(role: u8, base: f32) -> f32 {
    match role {
        ROLE_SCOUT => base * 1.4,
        ROLE_SOLDIER => base * 0.7,
        _ => base,
    }
}
