use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::{
    build::BuildOrder,
    config::Config,
    ease::ease_to_x_geometric,
    graphics::SpriteType,
    map::tile_center,
    tower::{create_tower, Tower, TowerStatus, FACTORY_INDEX},
    world::{Map, RenderState, World},
};

#[derive(Serialize, Deserialize)]
pub struct Factory {
    pub rotation: f32,
    pub rotation_speed: f32,
    pub is_constructing: bool,
}

const MAX_ROTATION_SPEED: f32 = 0.03;
const ACCELERATION: f32 = 0.0005;

impl RenderState {
    pub fn dump_factory(&mut self, row: usize, col: usize, rotation: f32, color: u32, alpha: f32) {
        let (x, y) = tile_center(row, col);
        self.sprite_data
            .push(SpriteType::TowerBase as u8, x, y, 0.0, alpha, color);
        self.sprite_data
            .push(SpriteType::Factory as u8, x, y, rotation, alpha, 0x000000);
    }
}

pub fn create_factory(
    entity: u32,
    row: usize,
    col: usize,
    towers: &mut Map<u32, Tower>,
    towers_by_pos: &mut Map<(usize, usize), u32>,
    factories: &mut Map<u32, Factory>,
    build_orders: &mut VecDeque<BuildOrder>,
    config: &Config,
) -> u32 {
    create_tower(
        row,
        col,
        entity,
        FACTORY_INDEX,
        towers,
        towers_by_pos,
        build_orders,
        config,
    );

    factories.insert(
        entity,
        Factory {
            rotation: 0.0,
            rotation_speed: 0.0,
            is_constructing: false,
        },
    );

    entity
}

impl World {
    pub fn dump_factories(&mut self) {
        for (entity, factory) in &self.core_state.factories {
            if let Some(tower) = self.core_state.towers.get(entity) {
                let alpha = if tower.status == TowerStatus::Queued {
                    0.5
                } else {
                    1.0
                };
                self.render_state.dump_factory(
                    tower.row,
                    tower.col,
                    factory.rotation,
                    self.config.common[FACTORY_INDEX].color,
                    alpha,
                );
            }
        }
    }

    pub fn rotate_factories(&mut self) {
        for factory in self.core_state.factories.values_mut() {
            if factory.is_constructing {
                ease_to_x_geometric(
                    &mut factory.rotation,
                    &mut factory.rotation_speed,
                    f32::INFINITY,
                    0.0,
                    MAX_ROTATION_SPEED,
                    ACCELERATION,
                    crate::ease::Domain::NumberLine,
                );
            } else {
                let curr_rotation = factory.rotation;
                let curr_speed = factory.rotation_speed;
                ease_to_x_geometric(
                    &mut factory.rotation,
                    &mut factory.rotation_speed,
                    curr_rotation,
                    curr_speed,
                    MAX_ROTATION_SPEED,
                    ACCELERATION,
                    crate::ease::Domain::NumberLine,
                );
            }
        }
    }
}
