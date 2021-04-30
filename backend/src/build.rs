use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    factory::create_factory,
    falcon::create_falcon_tower,
    graphics::SpriteType,
    map::{tile_center, Tile, TRUE_MAP_WIDTH},
    missile::create_missile_tower,
    swallow::create_swallow_tower,
    tower::{
        Tower, TowerStatus, FACTORY_INDEX, FALCON_INDEX, FIRE_INDEX, GAUSS_INDEX, MISSILE_INDEX,
        SWALLOW_INDEX, TESLA_INDEX, TREE_INDEX,
    },
    world::{Map, World},
};

/// Stores a tower/upgrade that we are building or want to build in the future.
#[derive(Serialize, Deserialize, Clone)]
pub struct BuildOrder {
    pub cost: u32,
    pub progress: f32,
    pub row: usize,
    pub col: usize,
    pub tower_entity: u32,
    pub build_type: BuildType,
}

impl BuildOrder {
    fn complete(&self, towers: &mut Map<u32, Tower>) {
        match self.build_type {
            BuildType::Tower => {
                if let Some(tower) = towers.get_mut(&self.tower_entity) {
                    tower.status = TowerStatus::Operational;
                }
            }
            BuildType::Upgrade { .. } => todo!(),
        }
    }

    /// Update the tower status when we begin construction
    fn notify_tower(&self, towers: &mut Map<u32, Tower>) {
        match self.build_type {
            BuildType::Tower => {
                if let Some(tower) = towers.get_mut(&self.tower_entity) {
                    tower.status = TowerStatus::Building;
                }
            }
            BuildType::Upgrade { .. } => todo!(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum BuildType {
    Tower,
    Upgrade {
        /// False iff the tower for this upgrade is still under initial construction.
        can_build: bool,
        upgrade_flag: u8,
    },
}

impl BuildType {
    fn can_build(&self) -> bool {
        match self {
            BuildType::Tower => true,
            BuildType::Upgrade { can_build, .. } => *can_build,
        }
    }
}

#[wasm_bindgen]
impl World {
    pub fn preview_build_tower(&mut self, row: usize, col: usize, tower_index: usize) {
        let base_tower = self.config.get_common(tower_index);
        self.render_state.preview_tower = Some(Tower {
            row,
            col,
            type_index: tower_index,
            range: base_tower.base_range,
            status: crate::tower::TowerStatus::Queued,
        });
    }

    pub fn hide_preview_tower(&mut self) {
        self.render_state.preview_tower = None;
    }

    pub fn queue_build_tower(&mut self, row: usize, col: usize, tower_index: usize) {
        // If there is a tower under construction in this spot already,
        // return and do nothing.
        for tower in self.core_state.towers.values() {
            if (row, col) == (tower.row, tower.col) && tower.status != TowerStatus::Queued {
                return;
            }
        }

        // Likewise, don't build on illegal terrain
        let true_row = row + 2;
        let true_col = col + 2;
        if Some(&Tile::Empty)
            != self
                .level_state
                .map
                .get(true_row * TRUE_MAP_WIDTH + true_col)
        {
            return;
        }

        // But if there is a tower build order in this spot, we replace it only
        // if it hasn't started yet. We do not replace upgrade build orders.
        let mut replace = None;
        for build_order in &self.core_state.build_queue {
            if (row, col) == (build_order.row, build_order.col) {
                match build_order.build_type {
                    BuildType::Tower if build_order.progress == 0.0 => {
                        replace = Some(build_order.tower_entity);
                        break;
                    }
                    _ => {
                        // Don't override existing in-progress construction
                        return;
                    }
                }
            }
        }

        // Now that we know for sure we will queue some construction, resume
        // the game if autopaused
        use crate::world::RunState;
        if let RunState::AutoPaused = self.run_state {
            self.run_state = RunState::Playing;
        }

        if let Some(entity) = replace {
            // By filtering out all orders with this (row, col), we
            // make sure we remove any upgrades queued for the tower in addition
            // to the tower itself.
            let build_queue = std::mem::take(&mut self.core_state.build_queue);
            for build_order in build_queue {
                if (row, col) != (build_order.row, build_order.col) {
                    self.core_state.build_queue.push_back(build_order);
                }
            }
            self.destroy_tower(entity);
        }

        if let Some(base_tower) = self.config.common.get(tower_index) {
            let cost = (base_tower.cost * 60.0) as u32;
            let tower_entity = self.create_specific_tower(tower_index, row, col);
            self.core_state.build_queue.push_back(BuildOrder {
                cost,
                progress: 0.0,
                row,
                col,
                build_type: BuildType::Tower,
                tower_entity,
            });
        }
    }

    pub fn queue_upgrade(&mut self, row: usize, col: usize, upgrade_index: usize) {}

    pub fn cancel_construction(&mut self, row: usize, col: usize) {
        let build_order = self
            .core_state
            .build_queue
            .iter()
            .enumerate()
            .find(|(_, build_order)| (row, col) == (build_order.row, build_order.col));
        if let Some((index, build_order)) = build_order {
            if let Some(&tower_entity) = self
                .core_state
                .towers_by_pos
                .get(&(build_order.row, build_order.col))
            {
                self.destroy_tower(tower_entity);
            }
            self.core_state.build_queue.remove(index);

            // Autopause
            if self.core_state.build_queue.is_empty() {
                use crate::world::RunState;
                if let RunState::Playing = self.run_state {
                    self.run_state = RunState::AutoPaused;
                }
            }
        }
    }
}

impl World {
    pub fn progress_build(&mut self) {
        // Ideally, we'd like the player to build a queued order that isn't
        // adjacent to a factory. That way, they aren't stealing work that
        // the factory could have done for free.
        // If all queued orders are adjacent to a factory, we build the first
        // build order. That's the most predictable choice.

        let mut found_non_adjacent_order = false;

        let mut completed_order_indeces = Vec::new();
        for (i, build_order) in self.core_state.build_queue.iter_mut().enumerate() {
            let north = (build_order.row.wrapping_sub(1), build_order.col);
            let west = (build_order.row, build_order.col.wrapping_sub(1));
            let south = (build_order.row + 1, build_order.col);
            let east = (build_order.row, build_order.col + 1);
            let towers_by_pos = &mut self.core_state.towers_by_pos;
            let towers = &mut self.core_state.towers;
            let factories = &mut self.core_state.factories;

            if build_order.progress >= build_order.cost as f32 {
                build_order.complete(towers);
                completed_order_indeces.push(i);
                continue;
            }

            // Add progress from factory construction
            let mut found_factory = false;
            [north, south, east, west]
                .iter()
                .filter_map(|pos| towers_by_pos.get(pos))
                .for_each(|entity| {
                    if let Some(factory) = factories.get_mut(entity) {
                        // Even queued factories should count. Ignoring them
                        // just because they don't exist yet feels bad from the
                        // player's perspective.
                        found_factory = true;
                        if let Some(Tower { status, .. }) = towers.get(entity) {
                            if *status == TowerStatus::Operational && !factory.is_constructing {
                                factory.is_constructing = true;
                                build_order.notify_tower(towers);
                                build_order.progress += 0.5;
                            }
                        }
                    }
                });

            // Add progress from player's construction
            if !found_non_adjacent_order && !found_factory && build_order.build_type.can_build() {
                found_non_adjacent_order = true;
                build_order.notify_tower(towers);
                build_order.progress += 1.0;
            }
        }

        // Autopause
        if !completed_order_indeces.is_empty()
            && completed_order_indeces.len() == self.core_state.build_queue.len()
        {
            use crate::world::RunState;
            if let RunState::Playing = self.run_state {
                self.run_state = RunState::AutoPaused;
            }
        }

        for i in completed_order_indeces.into_iter().rev() {
            self.core_state.build_queue.remove(i);
        }

        if !found_non_adjacent_order {
            if let Some(build_order) = self.core_state.build_queue.front_mut() {
                if build_order.progress >= build_order.cost as f32 {
                    build_order.complete(&mut self.core_state.towers);
                    self.core_state.build_queue.pop_front();
                } else if build_order.build_type.can_build() {
                    build_order.notify_tower(&mut self.core_state.towers);
                    build_order.progress += 1.0;
                }
            }
        }

        self.rotate_factories();
        for factory in self.core_state.factories.values_mut() {
            factory.is_constructing = false;
        }
    }

    pub fn destroy_tower(&mut self, entity: u32) {
        if let Some(tower) = self.core_state.towers.remove(&entity) {
            self.core_state
                .towers_by_pos
                .remove(&(tower.row, tower.col));
            match tower.type_index {
                i if i == SWALLOW_INDEX => {
                    if let Some(targeter) = self.core_state.swallow_targeters.remove(&entity) {
                        for entity in targeter.home_swallow_entities {
                            self.core_state.swallows.remove(&entity);
                            self.core_state.mobs.remove(&entity);
                        }
                    }
                }
                i if i == FALCON_INDEX => {}
                i if i == TESLA_INDEX => {}
                i if i == GAUSS_INDEX => {}
                i if i == FIRE_INDEX => {}
                i if i == MISSILE_INDEX => {
                    self.core_state.missile_spawners.remove(&entity);
                }
                i if i == TREE_INDEX => {}
                i if i == FACTORY_INDEX => {}
                _ => {}
            }
        }
    }

    fn create_specific_tower(&mut self, tower_index: usize, row: usize, col: usize) -> u32 {
        match tower_index {
            i if i == SWALLOW_INDEX => create_swallow_tower(
                &mut self.core_state.entity_ids,
                row,
                col,
                &mut self.core_state.towers,
                &mut self.core_state.towers_by_pos,
                &mut self.core_state.swallow_targeters,
                &mut self.core_state.swallows,
                &mut self.core_state.mobs,
                &mut self.core_state.build_queue,
                &self.config,
            ),
            i if i == FALCON_INDEX => create_falcon_tower(
                &mut self.core_state.entity_ids,
                row,
                col,
                &mut self.core_state.towers,
                &mut self.core_state.towers_by_pos,
                &mut self.core_state.falcons,
                &mut self.core_state.mobs,
                &mut self.core_state.build_queue,
                &self.config,
            ),
            i if i == TESLA_INDEX => 0,
            i if i == GAUSS_INDEX => 0,
            i if i == FIRE_INDEX => 0,
            i if i == MISSILE_INDEX => create_missile_tower(
                self.core_state.entity_ids.next(),
                row,
                col,
                &mut self.core_state.towers,
                &mut self.core_state.towers_by_pos,
                &mut self.core_state.missile_spawners,
                &mut self.core_state.build_queue,
                &self.config,
            ),
            i if i == TREE_INDEX => 0,
            i if i == FACTORY_INDEX => create_factory(
                self.core_state.entity_ids.next(),
                row,
                col,
                &mut self.core_state.towers,
                &mut self.core_state.towers_by_pos,
                &mut self.core_state.factories,
                &mut self.core_state.build_queue,
                &self.config,
            ),
            _ => 0,
        }
    }

    pub fn dump_preview_tower(&mut self) {
        if let Some(tower) = &self.render_state.preview_tower {
            let tint = self.config.get_common(tower.type_index).color;
            let (x, y) = tile_center(tower.row, tower.col);
            self.render_state
                .sprite_data
                .push(SpriteType::TowerBase as u8, x, y, 0.0, 0.5, tint);
        }
    }
}

// Behavior for building teslas: from the mouse, find the closest path tile.
// Then try and straddle the path tile with teslas. There are some situations
// where horizontal and vertical placement are both possible, so tiebreak
// using mouse velocity?
