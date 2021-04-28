use serde::{Deserialize, Serialize};

use crate::waves::Wave;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub common: Vec<TowerType>,
    pub waves: Vec<Wave>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct TowerType {
    pub name: String,
    pub base_damage: f32,
    pub base_rate_of_fire: f32,
    pub base_range: f32,
    pub cost: f32,
    pub description: String,
    pub flavor: String,
    pub color: u32,
}

// We often need a default value for TowerType because we prefer unwrap_or over
// an unwrap which could panic (given malicious input from the client).
// We can't easily use Default::default() because we need it by reference, and
// when typing it inline, it is a temporary value (it gets dropped too early to
// be put in a reference). So this is a const manual default value.

pub const DEFAULT_TOWER_TYPE: &TowerType = &TowerType {
    name: String::new(),
    base_damage: 0.0,
    base_rate_of_fire: 0.0,
    base_range: 0.0,
    cost: 0.0,
    description: String::new(),
    flavor: String::new(),
    color: 0,
};

impl Config {
    pub fn get_common(&self, i: usize) -> &TowerType {
        self.common.get(i).unwrap_or(DEFAULT_TOWER_TYPE)
    }
}
