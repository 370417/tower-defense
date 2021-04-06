//! Missiles and missile towers.

use std::f32::{consts::PI, INFINITY};

use crate::{
    ease::ease_to_x_geometric,
    explosion::spawn_explosion,
    graphics::{SpriteData, SpriteType},
    map::{tile_center, Constants},
    mob::Mob,
    smoke::spawn_smoke_trail,
    targeting::{find_target, Targeting, Threat, THREAT_DISTANCE},
    tower::{Range, Tower},
    walker::STANDARD_ENEMY_RADIUS,
    world::{Map, World},
};

const MAX_SPEED: f32 = 5.0;
const MAX_TURN_SPEED: f32 = 0.13;
const ACCELERATION: f32 = 0.31;
const ROTATION_ACCEL: f32 = 0.05;

const TOWER_MAX_TURN_SPEED: f32 = 0.05;
const TOWER_ROTATION_ACCEL: f32 = 0.0025;

const SLOW_TURN_DURATION: u32 = 300;

pub const MISSILE_WIDTH: f32 = 5.0;
pub const MISSILE_LENGTH: f32 = 10.0;

pub struct MissileSpawner {
    pub reload_cost: u32,
    pub rotation: f32,
    pub rotation_speed: f32,
    left_reload_countdown: u32,
    right_reload_countdown: u32,
}

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
}

pub fn create_missile_tower(
    entity: u32,
    row: usize,
    col: usize,
    towers: &mut Map<u32, Tower>,
    spawners: &mut Map<u32, MissileSpawner>,
) {
    towers.insert(
        entity,
        Tower {
            row,
            col,
            range: Range::Circle { radius: 200.0 },
        },
    );
    spawners.insert(
        entity,
        MissileSpawner {
            left_reload_countdown: 0,
            right_reload_countdown: 0,
            reload_cost: 60,
            rotation: -PI / 2.0,
            rotation_speed: 0.0,
        },
    );
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

    missiles.insert(entity, Missile::new(target, rotation));
    mobs.insert(
        entity,
        Mob::new(
            tower_x + reloading_signum * rotation.sin() * MISSILE_WIDTH * 0.5,
            tower_y - reloading_signum * rotation.cos() * MISSILE_WIDTH * 0.5,
        ),
    );
}

impl World {
    pub fn operate_missile_towers(&mut self) {
        for (entity, spawner) in &mut self.missile_spawners {
            if let Some(tower) = self.towers.get(entity) {
                let (tower_x, tower_y) = tile_center(tower.row, tower.col);

                let first_mob_in_range = find_target(
                    tower_x,
                    tower_y,
                    tower.range,
                    Targeting::First,
                    &self.walkers,
                    &self.mobs,
                    &self.map,
                    &self.dist_from_entrance,
                    &self.dist_from_exit,
                );

                let target_else_closest_mob = match first_mob_in_range {
                    None => find_target(
                        tower_x,
                        tower_y,
                        Range::Circle { radius: INFINITY },
                        Targeting::Close,
                        &self.walkers,
                        &self.mobs,
                        &self.map,
                        &self.dist_from_entrance,
                        &self.dist_from_exit,
                    ),
                    some => some,
                };

                let goal_rotation = match target_else_closest_mob {
                    Some((_, x, y)) => f32::atan2(y - tower_y, x - tower_x),
                    None => spawner.rotation,
                };

                ease_to_x_geometric(
                    &mut spawner.rotation,
                    &mut spawner.rotation_speed,
                    goal_rotation,
                    TOWER_MAX_TURN_SPEED,
                    TOWER_ROTATION_ACCEL,
                    crate::ease::Domain::Radian { miss_adjust: 1.0 },
                );

                if spawner.right_reload_countdown > 0 {
                    spawner.right_reload_countdown -= 1;
                } else if spawner.left_reload_countdown <= 0 {
                    if let Some((target, _, _)) = first_mob_in_range {
                        let entity = self.entity_ids.next();
                        spawn_missile(
                            entity,
                            target,
                            tower,
                            spawner.rotation,
                            1.0,
                            &mut self.missiles,
                            &mut self.mobs,
                        );
                        spawn_smoke_trail(&mut self.entity_ids, &mut self.smoke_trails, entity);
                        spawner.right_reload_countdown = spawner.reload_cost;
                    }
                }

                if spawner.left_reload_countdown > 0 {
                    spawner.left_reload_countdown -= 1;
                } else if spawner.right_reload_countdown <= 0 {
                    if let Some((target, _, _)) = first_mob_in_range {
                        let entity = self.entity_ids.next();
                        spawn_missile(
                            entity,
                            target,
                            tower,
                            spawner.rotation,
                            -1.0,
                            &mut self.missiles,
                            &mut self.mobs,
                        );
                        spawn_smoke_trail(&mut self.entity_ids, &mut self.smoke_trails, entity);
                        spawner.left_reload_countdown = spawner.reload_cost;
                    }
                }
            }
        }
    }

    /// Missile behavior: (subject to change)
    /// Starts at a standstill and accelerates toward target.
    /// Max speed is simulated with simple air resistance.
    /// Turn radius increases as speed increases, and the missile slows down
    /// if it needs to turn a large amount.
    pub fn fly_missiles(&mut self) {
        let mut trash = Vec::new();
        for (&entity, missile) in &mut self.missiles {
            missile.age += 1;
            if let Some(target_mob) = self.mobs.get(&missile.target) {
                // Copy target's coordinates to satisfy borrow checker
                let target_x = target_mob.x;
                let target_y = target_mob.y;

                if let Some(missile_mob) = self.mobs.get_mut(&entity) {
                    // Check for collision
                    // The tip of a missile extends out in front of its
                    // x,y position.
                    let target_radius = STANDARD_ENEMY_RADIUS;
                    let missile_tip_x =
                        missile_mob.x + MISSILE_LENGTH * 0.5 * missile.rotation.cos();
                    let missile_tip_y =
                        missile_mob.y + MISSILE_LENGTH * 0.5 * missile.rotation.sin();
                    let distance_squared = (target_x - missile_tip_x) * (target_x - missile_tip_x)
                        + (target_y - missile_tip_y) * (target_y - missile_tip_y);
                    if distance_squared < (target_radius * target_radius) as f32 {
                        spawn_explosion(
                            self.entity_ids.next(),
                            &mut self.explosions,
                            missile_tip_x,
                            missile_tip_y,
                            1.2 * f32::TILE_SIZE,
                        );
                        trash.push(entity);
                        continue;
                    } else if distance_squared < THREAT_DISTANCE * THREAT_DISTANCE {
                        // Add threat to mobs
                        self.threats.insert(missile.target, Threat {});
                    }

                    // Aim toward the target
                    let rotation = f32::atan2(target_y - missile_mob.y, target_x - missile_mob.x);

                    // Rotation is slower when the missile is young
                    let rotation_accel = if missile.age >= SLOW_TURN_DURATION {
                        missile.rotation_acceleration
                    } else {
                        missile.rotation_acceleration
                            * (0.2 + 0.8 * missile.age as f32 / SLOW_TURN_DURATION as f32)
                    };

                    ease_to_x_geometric(
                        &mut missile.rotation,
                        &mut missile.rotation_speed,
                        rotation,
                        missile.max_turn_speed,
                        rotation_accel,
                        crate::ease::Domain::Radian { miss_adjust: 0.95 },
                    );

                    missile.speed += missile.acceleration;
                    // Apply air resistance
                    missile.speed *= missile.max_speed / (missile.max_speed + missile.acceleration);

                    // Update position
                    missile_mob.x += missile.speed * missile.rotation.cos();
                    missile_mob.y += missile.speed * missile.rotation.sin();
                }
            }
        }
        for entity in trash {
            self.missiles.remove(&entity);
            self.mobs.remove(&entity);
        }
    }
}

impl Missile {
    fn new(target: u32, rotation: f32) -> Missile {
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
        }
    }

    pub fn dump(&self, id: &u32, data: &mut SpriteData, mobs: &Map<u32, Mob>, frame_fudge: f32) {
        if let Some(mob) = mobs.get(id) {
            data.push(
                SpriteType::Missile as u8,
                mob.x + frame_fudge * (mob.x - mob.old_x),
                mob.y + frame_fudge * (mob.y - mob.old_y),
                self.rotation,
                1.0,
                0x000000,
            );
        }
    }
}

impl MissileSpawner {
    pub fn dump(&self, id: &u32, data: &mut SpriteData, towers: &Map<u32, Tower>) {
        if let Some(tower) = towers.get(id) {
            let (tower_x, tower_y) = tile_center(tower.row, tower.col);
            let cos = self.rotation.cos();
            let sin = self.rotation.sin();

            data.push(
                SpriteType::TowerBase as u8,
                tower_x,
                tower_y,
                0.0,
                1.0,
                0xf5bec5,
            );

            let left_peek = 3.0 - 3.0 * self.left_reload_countdown as f32 / self.reload_cost as f32;
            let right_peek =
                3.0 - 3.0 * self.right_reload_countdown as f32 / self.reload_cost as f32;
            let recoil_amount = (3.0 - left_peek.min(right_peek)) / 3.0;

            data.push(
                SpriteType::Missile as u8,
                tower_x + MISSILE_WIDTH * 0.5 * sin + (right_peek - recoil_amount) * cos,
                tower_y - MISSILE_WIDTH * 0.5 * cos + (right_peek - recoil_amount) * sin,
                self.rotation,
                1.0,
                0x000000,
            );

            data.push(
                SpriteType::Missile as u8,
                tower_x - MISSILE_WIDTH * 0.5 * sin + (left_peek - recoil_amount) * cos,
                tower_y + MISSILE_WIDTH * 0.5 * cos + (left_peek - recoil_amount) * sin,
                self.rotation,
                1.0,
                0x000000,
            );

            data.push(
                SpriteType::MissileTower as u8,
                tower_x - recoil_amount * cos,
                tower_y - recoil_amount * sin,
                self.rotation,
                1.0,
                0x000000,
            )
        }
    }
}
