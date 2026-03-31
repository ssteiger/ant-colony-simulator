use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FoodSource {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub amount: f32,
    pub max_amount: f32,
}
