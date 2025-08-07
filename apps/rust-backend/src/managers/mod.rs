pub mod ant_behavior;
pub mod ant_spawn;
pub mod colony;
pub mod environment;
pub mod pheromone;
pub mod rendering;

pub use ant_behavior::*;
pub use ant_spawn::{spawn_ant_with_big_brain, ant_health_system, basic_ant_movement_system};
pub use colony::*;
pub use environment::*;
pub use pheromone::*;
pub use rendering::*; 