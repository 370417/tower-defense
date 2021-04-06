use std::f32::consts::PI;

use crate::{
    graphics::{SpriteData, SpriteType},
    map::{tile_center, true_row_col, Constants},
    mob::Mob,
    targeting::{find_target, Targeting, Threat},
    tower::{Range, Tower},
    walker::walk_tile,
    world::{EntityIds, Map, World},
};

const RISING_ACCEL: f32 = 0.17;
const MAX_HEIGHT: f32 = 5.0 * f32::TILE_SIZE;
const RANGE: f32 = 5.0 * f32::TILE_SIZE;
const COOLDOWN: u32 = 60;
const SOAR_HEIGHT: f32 = MAX_HEIGHT + 100.0;

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

pub enum FalconState {
    Rising,
    Diving,
    Migrating { rotation: f32 },
    Recovering { countdown: u32 },
}

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

    pub fn dump(&self, id: &u32, data: &mut SpriteData, mobs: &Map<u32, Mob>, frame_fudge: f32) {
        if let Some(mob) = mobs.get(id) {
            let height = self.height + frame_fudge * (self.height - self.old_height);
            data.push(
                SpriteType::Falcon as u8,
                mob.x + frame_fudge * (mob.x - mob.old_x),
                mob.y + frame_fudge * (mob.y - mob.old_y) - height,
                self.rotation(),
                match self.state {
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
    falcons: &mut Map<u32, Falcon>,
    mobs: &mut Map<u32, Mob>,
) {
    let tower_entity = entities.next();
    let falcon_entity = entities.next();

    let (x, y) = tile_center(row, col);

    towers.insert(
        tower_entity,
        Tower {
            row,
            col,
            range: Range::Circle { radius: RANGE },
        },
    );
    falcons.insert(falcon_entity, Falcon::new_rising(tower_entity));
    mobs.insert(falcon_entity, Mob::new(x, y));
}

impl World {
    pub fn fly_falcons(&mut self) {
        for (entity, falcon) in &mut self.falcons {
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
                            self.threats.insert(target, Threat {});
                        }
                    }
                    if falcon.height <= 0.0 {
                        if let Some(target) = falcon.target {
                            // Unmark the target
                            if let Some(indicator) = self.target_indicators.get_mut(&target) {
                                indicator.falcons = indicator.falcons.saturating_sub(1);
                            }

                            // Check to see if we have hit the target
                        }

                        if let Some(tower) = self.towers.get(&falcon.curr_tower) {
                            if let Some(falcon_mob) = self.mobs.get_mut(entity) {
                                let (x, y) = tile_center(tower.row, tower.col);
                                falcon_mob.x = x;
                                falcon_mob.y = y;
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
                                if let Some(target_mob) = self.mobs.get(&target) {
                                    let dive_time = (falcon.height / falcon.speed).ceil();

                                    let mut x = target_mob.x;
                                    let mut y = target_mob.y;
                                    let (true_row, true_col) = true_row_col(x, y);
                                    walk_tile(
                                        &self.map, true_row, true_col, &mut x, &mut y, dive_time,
                                    );

                                    if let Some(falcon_mob) = self.mobs.get_mut(entity) {
                                        falcon_mob.x = x;
                                        falcon_mob.y = y;
                                    }
                                }
                            } else {
                                falcon.old_height = old_height;
                            }
                        }
                        None => {
                            // Look for a target
                            if let Some(tower) = self.towers.get(&falcon.curr_tower) {
                                let (tower_x, tower_y) = tile_center(tower.row, tower.col);
                                if let Some((target, _x, _y)) = find_target(
                                    tower_x,
                                    tower_y,
                                    tower.range,
                                    Targeting::First,
                                    &self.walkers,
                                    &self.mobs,
                                    &self.map,
                                    &self.dist_from_entrance,
                                    &self.dist_from_exit,
                                ) {
                                    falcon.target = Some(target);
                                    falcon.speed = 0.0;
                                    falcon.accel = RISING_ACCEL;

                                    // Mark the target
                                    let indicator = self
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
}

fn falcon_fade(normalized_height: f32) -> f32 {
    let normalized_height = normalized_height.max(0.0).min(1.0);
    if normalized_height < 0.6 {
        0.0
    } else {
        (normalized_height - 0.6) / 0.4
    }
}
