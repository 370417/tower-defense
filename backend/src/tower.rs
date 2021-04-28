use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    build::{BuildOrder, BuildType},
    config::Config,
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
    config: &Config,
) {
    let base_tower = config.get_common(type_index);

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

pub const SWALLOW_INDEX: usize = 0;
pub const FALCON_INDEX: usize = 4;
pub const TESLA_INDEX: usize = 1;
pub const GAUSS_INDEX: usize = 5;
pub const FIRE_INDEX: usize = 2;
pub const MISSILE_INDEX: usize = 6;
pub const TREE_INDEX: usize = 3;
pub const FACTORY_INDEX: usize = 7;

#[wasm_bindgen]
impl World {
    pub fn query_tower_name(&self, tower_index: usize) -> String {
        self.config
            .common
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.name.clone()))
            .unwrap_or_default()
    }

    pub fn query_tower_base_cost(&self, tower_index: usize) -> f32 {
        self.config
            .common
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.cost))
            .unwrap_or_default()
    }

    pub fn query_tower_base_damage(&self, tower_index: usize) -> f32 {
        self.config
            .common
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.base_damage))
            .unwrap_or_default()
    }

    pub fn query_tower_base_rate_of_fire(&self, tower_index: usize) -> f32 {
        self.config
            .common
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.base_rate_of_fire))
            .unwrap_or_default()
    }

    pub fn query_tower_base_range(&self, tower_index: usize) -> f32 {
        self.config
            .common
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.base_range))
            .unwrap_or_default()
    }

    pub fn query_tower_description(&self, tower_index: usize) -> String {
        self.config
            .common
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.description.clone()))
            .unwrap_or_default()
    }

    pub fn query_tower_flavor(&self, tower_index: usize) -> String {
        self.config
            .common
            .get(tower_index)
            .and_then(|base_tower| Some(base_tower.flavor.clone()))
            .unwrap_or_default()
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
