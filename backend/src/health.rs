use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Health {
    pub curr_health: f32,
    pub max_health: f32,
}
