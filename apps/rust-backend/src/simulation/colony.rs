use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Colony {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub food_stored: f32,
    pub color_hue: u16,
}
