//! Missiles and missile towers.

use std::{
    collections::VecDeque,
    f32::consts::{PI, TAU},
};

use serde::{Deserialize, Serialize};

use crate::{
    build::BuildOrder,
    config::Config,
    ease::ease_to_x_geometric,
    explosion::spawn_explosion,
    graphics::SpriteType,
    map::{tile_center, Constants},
    mob::Mob,
    smoke::spawn_smoke_trail,
    targeting::{find_target, Targeting, Threat, THREAT_DISTANCE},
    tower::{create_tower, Tower, TowerStatus, MISSILE_INDEX},
    walker::STANDARD_ENEMY_RADIUS,
    world::{Map, World},
};

const MAX_SPEED: f32 = 5.0;
const MAX_TURN_SPEED: f32 = 0.13;
const ACCELERATION: f32 = 0.31;
const ROTATION_ACCEL: f32 = 0.05;

const TOWER_MAX_TURN_SPEED: f32 = 0.08;
const TOWER_ROTATION_ACCEL: f32 = 0.002;

pub const MISSILE_WIDTH: f32 = 5.0;
pub const MISSILE_LENGTH: f32 = 10.0;

#[derive(Serialize, Deserialize, Clone)]
pub struct MissileSpawner {
    pub reload_cost: u32,
    pub rotation: f32,
    pub rotation_speed: f32,
    left_reload_countdown: u32,
    right_reload_countdown: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Missile {
    pub target: u32,
    pub rotation: f32,
    pub rotation_speed: f32,
    pub rotation_acceleration: f32,
    pub max_turn_speed: f32,
    pub speed: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub age: u32,
    tower_x: f32,
    tower_y: f32,
}

pub fn create_missile_tower(
    entity: u32,
    row: usize,
    col: usize,
    towers: &mut Map<u32, Tower>,
    towers_by_pos: &mut Map<(usize, usize), u32>,
    spawners: &mut Map<u32, MissileSpawner>,
    build_orders: &mut VecDeque<BuildOrder>,
    config: &Config,
) -> u32 {
    create_tower(
        row,
        col,
        entity,
        MISSILE_INDEX,
        towers,
        towers_by_pos,
        build_orders,
        config,
    );
    spawners.insert(
        entity,
        MissileSpawner {
            left_reload_countdown: 0,
            right_reload_countdown: 0,
            reload_cost: (60.0 / config.common[MISSILE_INDEX].base_rate_of_fire).round() as u32,
            rotation: -PI / 2.0,
            rotation_speed: 0.0,
        },
    );
    entity
}

fn spawn_missile(
    entity: u32,
    target: u32,
    tower: &Tower,
    rotation: f32,
    reloading_signum: f32,
    missiles: &mut Map<u32, Missile>,
    mobs: &mut Map<u32, Mob>,
) {
    let (tower_x, tower_y) = tile_center(tower.row, tower.col);

    missiles.insert(entity, Missile::new(target, rotation, tower_x, tower_y));
    mobs.insert(
        entity,
        Mob::new(
            tower_x + reloading_signum * rotation.sin() * MISSILE_WIDTH * 0.5,
            tower_y - reloading_signum * rotation.cos() * MISSILE_WIDTH * 0.5,
        ),
    );
}

fn fly_toward(
    target_x: f32,
    target_y: f32,
    missile: &mut Missile,
    missile_mob: &mut Mob,
    // A positive value, measured in radians, of rotation to be added to the usual rotation.
    // A non-zero value here will cause the missile to circle its target instead of heading
    // directly at it.
    orbit_adjust: f32,
) {
    // Aim toward the target
    let rotation = f32::atan2(target_y - missile_mob.y, target_x - missile_mob.x) + orbit_adjust;

    // Scale turn speed by movement speed so that missiles aren't super
    // maneuverable right out the gate.
    let max_turn_speed = missile.max_turn_speed * missile.speed / missile.max_speed;

    ease_to_x_geometric(
        &mut missile.rotation,
        &mut missile.rotation_speed,
        rotation,
        0.0, // trying to use calc_sweep_angle here makes the missile miss more often
        max_turn_speed,
        missile.rotation_acceleration,
        crate::ease::Domain::Radian { miss_adjust: 0.95 },
    );

    missile.speed += missile.acceleration;
    // Apply air resistance
    missile.speed *= missile.max_speed / (missile.max_speed + missile.acceleration);

    // Update position
    missile_mob.x += missile.speed * missile.rotation.cos();
    missile_mob.y += missile.speed * missile.rotation.sin();
}

impl World {
    pub fn operate_missile_towers(&mut self) {
        for (entity, spawner) in &mut self.core_state.missile_spawners {
            if let Some(tower) = self.core_state.towers.get(entity) {
                if tower.status != TowerStatus::Operational {
                    continue;
                }

                let (tower_x, tower_y) = tile_center(tower.row, tower.col);

                let first_mob_in_range = find_target(
                    tower_x,
                    tower_y,
                    tower.range,
                    Targeting::First,
                    &self.core_state.walkers,
                    &self.core_state.mobs,
                    &self.level_state,
                );

                let target_else_closest_mob = match first_mob_in_range {
                    None => find_target(
                        tower_x,
                        tower_y,
                        f32::INFINITY,
                        Targeting::Close,
                        &self.core_state.walkers,
                        &self.core_state.mobs,
                        &self.level_state,
                    ),
                    some => some,
                };

                let (target_rotation, target_d_rotation) = match target_else_closest_mob {
                    Some((entity, x, y)) => {
                        if let Some(mob) = self.core_state.mobs.get(&entity) {
                            (
                                f32::atan2(y - tower_y, x - tower_x),
                                calc_target_sweep_angle(tower_x, tower_y, mob),
                            )
                        } else {
                            (spawner.rotation + 0.5 * spawner.rotation_speed, 0.0)
                        }
                    }
                    None => (spawner.rotation + 0.5 * spawner.rotation_speed, 0.0),
                };

                ease_to_x_geometric(
                    &mut spawner.rotation,
                    &mut spawner.rotation_speed,
                    target_rotation,
                    target_d_rotation,
                    TOWER_MAX_TURN_SPEED,
                    TOWER_ROTATION_ACCEL,
                    crate::ease::Domain::Radian { miss_adjust: 1.0 },
                );

                if spawner.right_reload_countdown > 0 {
                    spawner.right_reload_countdown -= 1;
                } else if spawner.left_reload_countdown <= 0 {
                    if let Some((target, _, _)) = first_mob_in_range {
                        let entity = self.core_state.entity_ids.next();
                        spawn_missile(
                            entity,
                            target,
                            tower,
                            spawner.rotation,
                            1.0,
                            &mut self.core_state.missiles,
                            &mut self.core_state.mobs,
                        );
                        spawn_smoke_trail(
                            &mut self.core_state.entity_ids,
                            &mut self.render_state.smoke_trails,
                            entity,
                        );
                        spawner.right_reload_countdown = spawner.reload_cost;
                    }
                }

                if spawner.left_reload_countdown > 0 {
                    spawner.left_reload_countdown -= 1;
                } else if spawner.right_reload_countdown <= 0 {
                    if let Some((target, _, _)) = first_mob_in_range {
                        let entity = self.core_state.entity_ids.next();
                        spawn_missile(
                            entity,
                            target,
                            tower,
                            spawner.rotation,
                            -1.0,
                            &mut self.core_state.missiles,
                            &mut self.core_state.mobs,
                        );
                        spawn_smoke_trail(
                            &mut self.core_state.entity_ids,
                            &mut self.render_state.smoke_trails,
                            entity,
                        );
                        spawner.left_reload_countdown = spawner.reload_cost;
                    }
                }
            }
        }
    }

    /// Missile behavior: (subject to change)
    /// Starts at a standstill and accelerates toward target.
    /// Max speed is simulated with simple air resistance.
    pub fn fly_missiles(&mut self) {
        let mut trash = Vec::new();
        for (&entity, missile) in &mut self.core_state.missiles {
            missile.age += 1;

            if let Some(missile_mob) = self.core_state.mobs.get(&entity) {
                for enemy_entity in self.core_state.walkers.keys() {
                    if let Some(enemy_mob) = self.core_state.mobs.get(enemy_entity) {
                        // Check for collision
                        // The tip of a missile extends out in front of its
                        // x,y position.
                        let target_radius = STANDARD_ENEMY_RADIUS;
                        let missile_tip_x =
                            missile_mob.x + MISSILE_LENGTH * 0.5 * missile.rotation.cos();
                        let missile_tip_y =
                            missile_mob.y + MISSILE_LENGTH * 0.5 * missile.rotation.sin();
                        let distance_squared = (enemy_mob.x - missile_tip_x)
                            * (enemy_mob.x - missile_tip_x)
                            + (enemy_mob.y - missile_tip_y) * (enemy_mob.y - missile_tip_y);
                        if distance_squared < (target_radius * target_radius) as f32 {
                            spawn_explosion(
                                self.core_state.entity_ids.next(),
                                &mut self.core_state.explosions,
                                missile_tip_x,
                                missile_tip_y,
                                1.2 * f32::TILE_SIZE,
                                missile.damage(&self.config),
                            );
                            trash.push(entity);
                            continue;
                        } else if distance_squared < THREAT_DISTANCE * THREAT_DISTANCE {
                            // Add threat to mobs
                            self.core_state.threats.insert(*enemy_entity, Threat {});
                        }
                    }
                }
            }

            if let Some(target_mob) = self.core_state.mobs.get(&missile.target) {
                let target_mob = target_mob.clone();

                if let Some(missile_mob) = self.core_state.mobs.get_mut(&entity) {
                    fly_toward(target_mob.x, target_mob.y, missile, missile_mob, 0.0);
                }
            } else {
                let (x, y) = if let Some(mob) = self.core_state.mobs.get(&entity) {
                    (mob.x, mob.y)
                } else {
                    (0.0, 0.0)
                };
                // Find a new target.
                // If there are enemies around, aim for the closest one.
                if let Some((target, target_x, target_y)) = find_target(
                    x,
                    y,
                    f32::INFINITY,
                    Targeting::Close,
                    &self.core_state.walkers,
                    &self.core_state.mobs,
                    &self.level_state,
                ) {
                    missile.target = target;
                    if let Some(missile_mob) = self.core_state.mobs.get_mut(&entity) {
                        fly_toward(target_x, target_y, missile, missile_mob, 0.0);
                    }
                } else {
                    // If there are no enemies, try and orbit the parent tower
                    if let Some(missile_mob) = self.core_state.mobs.get_mut(&entity) {
                        fly_toward(missile.tower_x, missile.tower_y, missile, missile_mob, 1.23);
                    }
                }
            }
        }
        for entity in trash {
            self.core_state.missiles.remove(&entity);
            self.core_state.mobs.remove(&entity);
        }
    }

    pub fn dump_missiles(&mut self, frame_fudge: f32) {
        for (entity, missile) in &self.core_state.missiles {
            if let Some(mob) = self.core_state.mobs.get(entity) {
                self.render_state.sprite_data.push(
                    SpriteType::Missile as u8,
                    mob.x + frame_fudge * (mob.x - mob.old_x),
                    mob.y + frame_fudge * (mob.y - mob.old_y),
                    missile.rotation,
                    1.0,
                    0x000000,
                );
            }
        }
    }

    pub fn dump_missile_towers(&mut self, frame_fudge: f32) {
        for (entity, spawner) in &self.core_state.missile_spawners {
            if let Some(tower) = self.core_state.towers.get(entity) {
                let (tower_x, tower_y) = tile_center(tower.row, tower.col);

                let rotation = spawner.rotation + spawner.rotation_speed * frame_fudge;

                let cos = rotation.cos();
                let sin = rotation.sin();

                self.render_state.sprite_data.push(
                    SpriteType::TowerBase as u8,
                    tower_x,
                    tower_y,
                    0.0,
                    1.0,
                    self.config.common[MISSILE_INDEX].color,
                );

                let left_peek = 3.0
                    * ease_peek(spawner.left_reload_countdown as f32 / spawner.reload_cost as f32);
                let right_peek = 3.0
                    * ease_peek(spawner.right_reload_countdown as f32 / spawner.reload_cost as f32);

                let recoil_amount = 2.0
                    * spawner
                        .left_reload_countdown
                        .max(spawner.right_reload_countdown) as f32
                    / spawner.reload_cost as f32;

                self.render_state.sprite_data.push(
                    SpriteType::Missile as u8,
                    tower_x + MISSILE_WIDTH * 0.5 * sin + (right_peek - recoil_amount) * cos,
                    tower_y - MISSILE_WIDTH * 0.5 * cos + (right_peek - recoil_amount) * sin,
                    rotation,
                    1.0,
                    0x000000,
                );

                self.render_state.sprite_data.push(
                    SpriteType::Missile as u8,
                    tower_x - MISSILE_WIDTH * 0.5 * sin + (left_peek - recoil_amount) * cos,
                    tower_y + MISSILE_WIDTH * 0.5 * cos + (left_peek - recoil_amount) * sin,
                    rotation,
                    1.0,
                    0x000000,
                );

                self.render_state.sprite_data.push(
                    SpriteType::MissileTower as u8,
                    tower_x - recoil_amount * cos,
                    tower_y - recoil_amount * sin,
                    rotation,
                    1.0,
                    self.config.common[MISSILE_INDEX].color,
                );
            }
        }
    }
}

impl Missile {
    fn new(target: u32, rotation: f32, tower_x: f32, tower_y: f32) -> Missile {
        Missile {
            target,
            rotation,
            rotation_speed: 0.0,
            rotation_acceleration: ROTATION_ACCEL,
            max_turn_speed: MAX_TURN_SPEED,
            max_speed: MAX_SPEED,
            speed: 0.0,
            acceleration: ACCELERATION,
            age: 0,
            tower_x,
            tower_y,
        }
    }

    fn damage(&self, config: &Config) -> f32 {
        config.common[MISSILE_INDEX].base_damage
    }
}

fn ease_peek(t: f32) -> f32 {
    if t > 0.8 {
        0.0
    } else if t < 0.2 {
        1.0
    } else {
        1.0 - (t - 0.2) / 0.6
    }
}

fn calc_target_sweep_angle(self_x: f32, self_y: f32, target_mob: &Mob) -> f32 {
    let old_angle = f32::atan2(target_mob.old_y - self_y, target_mob.old_x - self_x);
    let new_angle = f32::atan2(target_mob.y - self_y, target_mob.x - self_x);
    let d_angle = new_angle - old_angle;
    if d_angle > PI {
        d_angle - TAU
    } else if d_angle < -PI {
        d_angle + TAU
    } else {
        d_angle
    }
}
