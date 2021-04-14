use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    build::{BuildOrder, BuildType},
    map::Constants,
    world::{Map, World},
};

#[derive(Serialize, Deserialize)]
pub struct Tower {
    pub row: usize,
    pub col: usize,
    pub range: f32,
    pub type_index: usize,
    pub status: TowerStatus,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum TowerStatus {
    Queued,
    Building,
    Operational,
    Upgrading,
}

/// Do the three things that any tower creation needs:
/// 1. add a Tower struct to the towers components
/// 2. add it to the tower spatial index
/// 3. allow any upgrades on this tower to make build progress
pub fn create_tower(
    row: usize,
    col: usize,
    entity: u32,
    type_index: usize,
    towers: &mut Map<u32, Tower>,
    towers_by_pos: &mut Map<(usize, usize), u32>,
    build_orders: &mut VecDeque<BuildOrder>,
) {
    let base_tower = BASE_TOWERS.get(type_index).unwrap_or(&BASE_TOWERS[0]);

    towers.insert(
        entity,
        Tower {
            row,
            col,
            range: base_tower.base_range * f32::TILE_SIZE,
            type_index,
            status: TowerStatus::Queued,
        },
    );
    towers_by_pos.insert((row, col), entity);
    for build_order in build_orders {
        if let BuildType::Upgrade {
            ref mut can_build, ..
        } = build_order.build_type
        {
            *can_build = true;
        }
    }
}

pub fn build_towers_by_pos(towers: &Map<u32, Tower>) -> Map<(usize, usize), u32> {
    towers
        .iter()
        .map(|(entity, tower)| ((tower.row, tower.col), *entity))
        .collect()
}

pub struct TowerType {
    pub name: &'static str,
    pub base_damage: f32,
    pub base_rate_of_fire: f32,
    pub base_range: f32,
    pub cost: f32,
    pub description: &'static str,
    pub flavor: &'static str,
    pub color: u32,
}

pub const SWALLOW_INDEX: usize = 0;
pub const FALCON_INDEX: usize = 4;
pub const TESLA_INDEX: usize = 1;
pub const GAUSS_INDEX: usize = 5;
pub const FIRE_INDEX: usize = 2;
pub const MISSILE_INDEX: usize = 6;
pub const TREE_INDEX: usize = 3;
pub const FACTORY_INDEX: usize = 7;

pub const BASE_TOWERS: [TowerType; 8] = [
    TowerType {
        name: "Swallow",
        base_damage: 1.0,
        base_rate_of_fire: 97.5,
        base_range: 3.6,
        cost: 3.0,
        description: "Attacks faster as enemies get closer.",
        flavor: "",
        color: 0xd4e8ee,
    },
    TowerType {
        name: "Tesla",
        base_damage: 1.0,
        base_rate_of_fire: 1.0,
        base_range: 1.0,
        cost: 3.5,
        description: "Generates lightning between pairs of towers.",
        flavor: "",
        color: 0xeedcba,
    },
    TowerType {
        name: "Fire",
        base_damage: 1.0,
        base_rate_of_fire: f32::INFINITY,
        base_range: 1.0,
        cost: 4.0,
        description: "Deals damage over time.",
        flavor: "“Build a man a fire, and he'll be warm for a day.”",
        color: 0xf5bec5,
    },
    TowerType {
        name: "Tree",
        base_damage: 1.0,
        base_rate_of_fire: 1.0,
        base_range: 1.0,
        cost: 5.0,
        description: "Slows and roots all enemies in range.",
        flavor: "",
        color: 0xc0e6bf,
    },
    TowerType {
        name: "Falcon",
        base_damage: 1.0,
        base_rate_of_fire: 1.0,
        base_range: 6.0,
        cost: 6.0,
        description: "Dives down and scatters nearby enemies.",
        flavor: "Frightful.",
        color: 0xd4e8ee,
    },
    TowerType {
        name: "Gauss",
        base_damage: 1.0,
        base_rate_of_fire: 1.0,
        base_range: 1.0,
        cost: 7.0,
        description: "Fires in a fixed direction. Can be chained end-to-end.",
        flavor: "“Theory attracts practice as the magnet attracts iron.”",
        color: 0xeedcba,
    },
    TowerType {
        name: "Missile",
        base_damage: 1.0,
        base_rate_of_fire: 1.0,
        base_range: 6.4,
        cost: 8.0,
        description: "Fires missiles that deal splash damage.",
        flavor: "Anti-ninja technology.",
        color: 0xf5bec5,
    },
    TowerType {
        name: "Factory",
        base_damage: 0.0,
        base_rate_of_fire: 0.0,
        base_range: 1.0,
        cost: 20.0,
        description: "Helps build and upgrade adjacent towers.",
        flavor: "",
        color: 0xc0e6bf,
    },
];

#[wasm_bindgen]
impl World {
    pub fn query_tower_name(&self, tower_index: usize) -> String {
        BASE_TOWERS
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.name))
            .unwrap_or_default()
            .to_owned()
    }

    pub fn query_tower_base_cost(&self, tower_index: usize) -> f32 {
        BASE_TOWERS
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.cost))
            .unwrap_or_default()
    }

    pub fn query_tower_base_damage(&self, tower_index: usize) -> f32 {
        BASE_TOWERS
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.base_damage))
            .unwrap_or_default()
    }

    pub fn query_tower_base_rate_of_fire(&self, tower_index: usize) -> f32 {
        BASE_TOWERS
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.base_rate_of_fire))
            .unwrap_or_default()
    }

    pub fn query_tower_base_range(&self, tower_index: usize) -> f32 {
        BASE_TOWERS
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.base_range))
            .unwrap_or_default()
    }

    pub fn query_tower_description(&self, tower_index: usize) -> String {
        BASE_TOWERS
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.description))
            .unwrap_or_default()
            .to_owned()
    }

    pub fn query_tower_flavor(&self, tower_index: usize) -> String {
        BASE_TOWERS
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.flavor))
            .unwrap_or_default()
            .to_owned()
    }

    // what about towers in progress?
    pub fn query_tower_entity(&self, row: usize, col: usize) -> u32 {
        *self.core_state.towers_by_pos.get(&(row, col)).unwrap_or(&0)
    }

    // 'queued' | 'building' | 'operational' | 'upgrading'
    pub fn query_tower_status(&self, tower_entity: u32) -> String {
        self.core_state
            .towers
            .get(&tower_entity)
            .and_then(|tower| match tower.status {
                TowerStatus::Building => Some("building"),
                TowerStatus::Operational => Some("operational"),
                TowerStatus::Queued => Some("queued"),
                TowerStatus::Upgrading => Some("upgrading"),
            })
            .unwrap_or_default()
            .to_owned()
    }
}
