use std::{collections::VecDeque, f32::consts::PI};

use serde::{Deserialize, Serialize};

use crate::{
    build::BuildOrder,
    config::Config,
    graphics::{SpriteData, SpriteType},
    map::{tile_center, true_row_col, Constants},
    mob::Mob,
    targeting::{find_target, Targeting, Threat},
    tower::{create_tower, Tower, TowerStatus, FALCON_INDEX},
    walker::walk_tile,
    world::{EntityIds, Map, World},
};

const RISING_ACCEL: f32 = 0.17;
const MAX_HEIGHT: f32 = 5.0 * f32::TILE_SIZE;
const COOLDOWN: u32 = 60;
const SOAR_HEIGHT: f32 = MAX_HEIGHT + 100.0;

#[derive(Serialize, Deserialize, Clone)]
pub struct Falcon {
    pub speed: f32,
    pub accel: f32,
    pub old_height: f32,
    pub height: f32,
    pub state: FalconState,
    pub target: Option<u32>,
    pub home_tower: u32,
    pub curr_tower: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum FalconState {
    Rising,
    Diving,
    Migrating { rotation: f32 },
    Recovering { countdown: u32 },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TargetIndicator {
    // Like a reference count but for falcons instead of pointers :)
    pub falcons: u32,
}

impl Falcon {
    fn new_rising(tower: u32) -> Falcon {
        Falcon {
            speed: 0.0,
            accel: 0.0,
            height: 0.0,
            old_height: 0.0,
            state: FalconState::Rising,
            target: None,
            home_tower: tower,
            curr_tower: tower,
        }
    }

    fn rotation(&self) -> f32 {
        match self.state {
            FalconState::Rising | FalconState::Recovering { .. } => -PI / 2.0,
            FalconState::Diving => PI / 2.0,
            FalconState::Migrating { rotation } => rotation,
        }
    }
}

impl TargetIndicator {
    pub fn dump(&self, id: &u32, data: &mut SpriteData, mobs: &Map<u32, Mob>, frame_fudge: f32) {
        if self.falcons == 0 {
            return;
        }
        if let Some(mob) = mobs.get(id) {
            data.push(
                SpriteType::Indicator as u8,
                mob.x + frame_fudge * (mob.x - mob.old_x),
                mob.y + frame_fudge * (mob.y - mob.old_y) - 0.5 * f32::TILE_SIZE,
                0.0,
                1.0,
                0xffffff,
            );
        }
    }
}

pub fn create_falcon_tower(
    entities: &mut EntityIds,
    row: usize,
    col: usize,
    towers: &mut Map<u32, Tower>,
    towers_by_pos: &mut Map<(usize, usize), u32>,
    falcons: &mut Map<u32, Falcon>,
    mobs: &mut Map<u32, Mob>,
    build_orders: &mut VecDeque<BuildOrder>,
    config: &Config,
) -> u32 {
    let tower_entity = entities.next();

    let (x, y) = tile_center(row, col);

    create_tower(
        row,
        col,
        tower_entity,
        FALCON_INDEX,
        towers,
        towers_by_pos,
        build_orders,
        config,
    );

    falcons.insert(tower_entity, Falcon::new_rising(tower_entity));
    mobs.insert(tower_entity, Mob::new(x, y));

    tower_entity
}

impl World {
    pub fn fly_falcons(&mut self) {
        for (entity, falcon) in &mut self.core_state.falcons {
            match falcon.state {
                FalconState::Recovering { countdown } => {
                    falcon.state = if countdown == 0 {
                        FalconState::Rising
                    } else {
                        FalconState::Recovering {
                            countdown: countdown - 1,
                        }
                    }
                }
                FalconState::Diving => {
                    let old_height = falcon.height;
                    falcon.height -= falcon.speed;
                    falcon.old_height = old_height;
                    if falcon.height < 0.75 * MAX_HEIGHT {
                        // Alert the target
                        if let Some(target) = falcon.target {
                            self.core_state.threats.insert(target, Threat {});
                        }
                    }
                    if falcon.height <= 0.0 {
                        if let Some(target) = falcon.target {
                            // Unmark the target
                            if let Some(indicator) =
                                self.core_state.target_indicators.get_mut(&target)
                            {
                                indicator.falcons = indicator.falcons.saturating_sub(1);
                            }

                            // Check to see if we have hit the target
                        }

                        if let Some(tower) = self.core_state.towers.get(&falcon.curr_tower) {
                            if let Some(falcon_mob) = self.core_state.mobs.get_mut(entity) {
                                let (x, y) = tile_center(tower.row, tower.col);
                                falcon_mob.x = x;
                                falcon_mob.y = y;
                                // setting old_x/old_y avoids visual teleporting if we render
                                // in between frames
                                falcon_mob.old_x = x;
                                falcon_mob.old_y = y;
                            }
                        }
                        falcon.height = 0.0;
                        falcon.old_height = 0.0;
                        falcon.state = FalconState::Recovering {
                            countdown: COOLDOWN,
                        };
                        falcon.target = None;
                    }
                }
                FalconState::Rising => {
                    match falcon.target {
                        Some(target) => {
                            // Rise until MAX_HEIGHT
                            let old_height = falcon.height;
                            falcon.height += falcon.speed;
                            falcon.speed += RISING_ACCEL;
                            if falcon.height >= SOAR_HEIGHT {
                                // Start diving
                                falcon.state = FalconState::Diving;
                                falcon.height = MAX_HEIGHT;
                                // falcon.speed += EXTRA_DIVE_SPEED;
                                falcon.old_height = MAX_HEIGHT + falcon.speed;
                                falcon.accel = 0.0;

                                // Predict where the target will be at time of impact
                                if let Some(target_mob) = self.core_state.mobs.get(&target) {
                                    let dive_time = (falcon.height / falcon.speed).ceil();

                                    let mut x = target_mob.x;
                                    let mut y = target_mob.y;
                                    let (true_row, true_col) = true_row_col(x, y);
                                    walk_tile(
                                        &self.level_state.map,
                                        true_row,
                                        true_col,
                                        &mut x,
                                        &mut y,
                                        dive_time,
                                    );

                                    if let Some(falcon_mob) = self.core_state.mobs.get_mut(entity) {
                                        falcon_mob.x = x;
                                        falcon_mob.y = y;
                                        falcon_mob.old_x = x;
                                        falcon_mob.old_y = y;
                                    }
                                }
                            } else {
                                falcon.old_height = old_height;
                            }
                        }
                        None => {
                            if let Some(tower) = self.core_state.towers.get(entity) {
                                if tower.status != TowerStatus::Operational {
                                    return;
                                }
                            }
                            // Look for a target
                            if let Some(tower) = self.core_state.towers.get(&falcon.curr_tower) {
                                let (tower_x, tower_y) = tile_center(tower.row, tower.col);
                                if let Some((target, _x, _y)) = find_target(
                                    tower_x,
                                    tower_y,
                                    tower.range,
                                    Targeting::First,
                                    &self.core_state.walkers,
                                    &self.core_state.mobs,
                                    &self.level_state,
                                ) {
                                    falcon.target = Some(target);
                                    falcon.speed = 0.0;
                                    falcon.accel = RISING_ACCEL;

                                    // Mark the target
                                    let indicator = self
                                        .core_state
                                        .target_indicators
                                        .entry(target)
                                        .or_insert(TargetIndicator { falcons: 0 });
                                    indicator.falcons += 1;

                                    continue;
                                }
                            }
                            // Otherwise, look to migrate
                        }
                    }
                }
                FalconState::Migrating { rotation } => {}
            }
        }
    }

    pub fn dump_falcons(&mut self, frame_fudge: f32) {
        for (entity, falcon) in &mut self.core_state.falcons {
            let mob = self.core_state.mobs.get(entity);
            let tower = self.core_state.towers.get(entity);
            if let (Some(mob), Some(tower)) = (mob, tower) {
                let (tower_x, tower_y) = tile_center(tower.row, tower.col);
                self.render_state.sprite_data.push(
                    SpriteType::TowerBase as u8,
                    tower_x,
                    tower_y,
                    0.0,
                    1.0,
                    self.config.common[FALCON_INDEX].color,
                );

                let height = falcon.height + frame_fudge * (falcon.height - falcon.old_height);
                self.render_state.sprite_data.push(
                    SpriteType::Falcon as u8,
                    mob.x + frame_fudge * (mob.x - mob.old_x),
                    mob.y + frame_fudge * (mob.y - mob.old_y) - height,
                    falcon.rotation(),
                    match falcon.state {
                        FalconState::Recovering { countdown } => {
                            1.0 - countdown as f32 / COOLDOWN as f32
                        }
                        _ => 1.0 - falcon_fade(height / MAX_HEIGHT),
                    },
                    0x000000,
                );
            }
        }
    }
}

/// Return a falcon's opacity/alpha as a function of height.
fn falcon_fade(normalized_height: f32) -> f32 {
    let normalized_height = normalized_height.max(0.0).min(1.0);
    if normalized_height < 0.6 {
        0.0
    } else {
        (normalized_height - 0.6) / 0.4
    }
}
