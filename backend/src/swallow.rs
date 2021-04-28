//! Swallows and swallow towers
//!
//! Unlike missile towers, swallow towers take no part in targeting or spawning
//! projectiles, since the swallow is a persistent projectile.

use std::{
    collections::VecDeque,
    f32::consts::{PI, TAU},
};

use float_ord::FloatOrd;
use serde::{Deserialize, Serialize};

use crate::{
    build::BuildOrder,
    collision::circle_line_intersection,
    config::Config,
    ease::ease_to_x_geometric,
    graphics::{SpriteData, SpriteType},
    map::{tile_center, Constants},
    mob::{closest_walker, Mob},
    targeting::{find_target, Targeting, Threat},
    tower::{create_tower, Tower, TowerStatus, SWALLOW_INDEX},
    walker::STANDARD_ENEMY_RADIUS,
    world::{EntityIds, Map, World},
};

const SPEED: f32 = 5.2;
const ROTATION_ACCEL: f32 = 0.05;
const MAX_TURN_SPEED: f32 = 0.25;
const SWALLOW_RADIUS: f32 = f32::TILE_SIZE * 0.3;

const AFTER_IMAGE_PERIOD: u32 = 5;
const AFTER_IMAGE_DURATION: u32 = 33;

#[derive(Serialize, Deserialize)]
pub struct Swallow {
    pub target: Target,
    pub rotation: f32,
    pub rotation_speed: f32,
    pub rotation_accel: f32,
    pub max_turn_speed: f32,
    pub speed: f32,
    pub vanishing_x: f32,
    pub vanishing_y: f32,
    pub home_tower: u32,
    pub curr_tower: u32,
    pub after_image_countdown: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SwallowAfterImage {
    x: f32,
    y: f32,
    rotation: f32,
    age: u32,
}

#[derive(Serialize, Deserialize)]
/// Component for tower entities. Keeps track of the closest walker and only
/// updates when necessary. This way, if 100 swallows need to scan 100 towers
/// to see if they have an enemy in range, and all 100 swallows have different
/// ranges, we can avoid looping over all enemies over and over.
pub struct SwallowTargeter {
    pub closest_distance_squared: f32,
    pub closest_x: f32,
    pub closest_y: f32,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Target {
    Enemy(u32),
    Tower {
        /// Store distance so that we know when we have reached a local minimum.
        /// The usual technique of having a threshold for distance collision
        /// isn't reliable in this situation without making it unreasonably large.
        dist_from_tower_squared: f32,
        /// Total rotation, in radians. We keep track of this because swallows
        /// can fall into orbit around towers. After one full circle,
        /// we can break the orbit by increasing turn speed.
        net_rotation: f32,
    },
    None,
}

impl Swallow {
    fn new(rotation: f32, tower_x: f32, tower_y: f32, home_tower: u32) -> Swallow {
        Swallow {
            target: Target::None,
            rotation,
            rotation_speed: 0.0,
            rotation_accel: ROTATION_ACCEL,
            max_turn_speed: MAX_TURN_SPEED,
            speed: SPEED,
            vanishing_x: tower_x,
            vanishing_y: tower_y,
            home_tower,
            curr_tower: home_tower,
            after_image_countdown: 0,
        }
    }

    pub fn dump(&self, id: &u32, data: &mut SpriteData, mobs: &Map<u32, Mob>, frame_fudge: f32) {
        if let Some(mob) = mobs.get(id) {
            data.push(
                SpriteType::Swallow as u8,
                mob.x + frame_fudge * (mob.x - mob.old_x),
                mob.y + frame_fudge * (mob.y - mob.old_y),
                self.rotation,
                1.0,
                0x000000,
            );
        }
    }
}

impl SwallowAfterImage {
    pub fn dump(&self, data: &mut SpriteData, frame_fudge: f32) {
        data.push(
            SpriteType::Swallow as u8,
            self.x,
            self.y,
            self.rotation,
            0.2 - 0.2 * (self.age as f32 + frame_fudge) / AFTER_IMAGE_DURATION as f32,
            0x000000,
        );
    }
}

impl SwallowTargeter {
    fn is_active(&self, range: f32) -> bool {
        self.closest_distance_squared < range * range
    }

    pub fn dump(&self, id: &u32, data: &mut SpriteData, towers: &Map<u32, Tower>, config: &Config) {
        if let Some(tower) = towers.get(id) {
            let (tower_x, tower_y) = tile_center(tower.row, tower.col);

            data.push(
                SpriteType::TowerBase as u8,
                tower_x,
                tower_y,
                0.0,
                1.0,
                config.common[SWALLOW_INDEX].color,
            );

            if tower.status == TowerStatus::Queued {
                data.push(
                    SpriteType::TowerBase as u8,
                    tower_x,
                    tower_y,
                    0.0,
                    0.5,
                    0xffffff,
                );
            }
        }
    }
}

pub fn create_swallow_tower(
    entities: &mut EntityIds,
    row: usize,
    col: usize,
    towers: &mut Map<u32, Tower>,
    towers_by_pos: &mut Map<(usize, usize), u32>,
    swallow_targeters: &mut Map<u32, SwallowTargeter>,
    swallows: &mut Map<u32, Swallow>,
    mobs: &mut Map<u32, Mob>,
    build_orders: &mut VecDeque<BuildOrder>,
    config: &Config,
) -> u32 {
    let tower_entity = entities.next();
    let swallow_entity = entities.next();

    let (x, y) = tile_center(row, col);

    create_tower(
        row,
        col,
        tower_entity,
        SWALLOW_INDEX,
        towers,
        towers_by_pos,
        build_orders,
        config,
    );

    swallow_targeters.insert(
        tower_entity,
        SwallowTargeter {
            closest_distance_squared: f32::INFINITY,
            closest_x: 0.0,
            closest_y: 0.0,
        },
    );

    swallows.insert(swallow_entity, Swallow::new(-PI / 2.0, x, y, tower_entity));
    mobs.insert(swallow_entity, Mob::new(x, y));

    tower_entity
}

fn create_swallow_after_image(
    entity: u32,
    swallow_mob: &Mob,
    swallow: &Swallow,
    swallow_after_images: &mut Map<u32, SwallowAfterImage>,
) {
    swallow_after_images.insert(
        entity,
        SwallowAfterImage {
            x: swallow_mob.x,
            y: swallow_mob.y,
            rotation: swallow.rotation,
            age: 0,
        },
    );
}

fn migrate_to_tower(tower_entity: u32, tower: &Tower, swallow: &mut Swallow, swallow_mob: &Mob) {
    swallow.curr_tower = tower_entity;
    let (tower_x, tower_y) = tile_center(tower.row, tower.col);
    let dx = tower_x - swallow_mob.x;
    let dy = tower_y - swallow_mob.y;
    swallow.target = Target::Tower {
        dist_from_tower_squared: dx * dx + dy * dy,
        net_rotation: 0.0,
    };
    swallow.rotation = f32::atan2(dy, dx);
}

fn should_migrate_home(
    swallow: &mut Swallow,
    home_tower: &Tower,
    swallow_targeters: &Map<u32, SwallowTargeter>,
) -> bool {
    swallow.curr_tower != swallow.home_tower
        && swallow_targeters
            .get(&swallow.home_tower)
            .and_then(|targeter| Some(targeter.is_active(home_tower.range)))
            .unwrap_or(false)
}

impl World {
    pub fn fly_swallows(&mut self) {
        for (&entity, swallow) in &mut self.core_state.swallows {
            if let Some(swallow_mob) = self.core_state.mobs.get(&entity) {
                if let Some(home_tower) = self.core_state.towers.get(&swallow.home_tower) {
                    if home_tower.status != TowerStatus::Operational {
                        if swallow.home_tower == swallow.curr_tower
                            && swallow.target == Target::None
                        {
                            // If the swallow is chilling at home, don't move at all.
                            continue;
                        }
                        migrate_to_tower(swallow.home_tower, home_tower, swallow, swallow_mob);
                    }
                }
                match swallow.target {
                    Target::None => {
                        swallow.after_image_countdown =
                            swallow.after_image_countdown.saturating_sub(1);

                        if let Some(home_tower) = self.core_state.towers.get(&swallow.home_tower) {
                            if should_migrate_home(
                                swallow,
                                home_tower,
                                &self.core_state.swallow_targeters,
                            ) {
                                migrate_to_tower(
                                    swallow.home_tower,
                                    home_tower,
                                    swallow,
                                    swallow_mob,
                                );
                            } else if let Some((target, target_x, target_y)) = find_target(
                                swallow_mob.x,
                                swallow_mob.y,
                                home_tower.range,
                                Targeting::Close,
                                &self.core_state.walkers,
                                &self.core_state.mobs,
                                &self.level_state,
                            ) {
                                swallow.target = Target::Enemy(target);
                                swallow.rotation =
                                    f32::atan2(target_y - swallow_mob.y, target_x - swallow_mob.x);
                                swallow.vanishing_x = swallow_mob.x;
                                swallow.vanishing_y = swallow_mob.y;
                            } else if swallow.curr_tower != swallow.home_tower {
                                migrate_to_tower(
                                    swallow.home_tower,
                                    home_tower,
                                    swallow,
                                    swallow_mob,
                                );
                            } else {
                                let towers = &self.core_state.towers;
                                let closest_active_tower = self
                                    .core_state
                                    .swallow_targeters
                                    .iter()
                                    .filter(|(entity, targeter)| {
                                        targeter.is_active(home_tower.range)
                                            && **entity != swallow.curr_tower
                                    })
                                    .map(|(entity, _)| entity)
                                    .min_by_key(|entity| {
                                        towers.get(*entity).and_then(|tower| {
                                            let (tower_x, tower_y) =
                                                tile_center(tower.row, tower.col);
                                            let dx = tower_x - swallow_mob.x;
                                            let dy = tower_y - swallow_mob.y;
                                            Some(FloatOrd(dx * dx + dy * dy))
                                        })
                                    });

                                if let Some(target_tower) = closest_active_tower {
                                    if let Some(tower) = towers.get(target_tower) {
                                        migrate_to_tower(
                                            *target_tower,
                                            tower,
                                            swallow,
                                            swallow_mob,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Target::Enemy(target) => {
                        if let Some(target_mob) = self.core_state.mobs.get(&target) {
                            // Copy target's coordinates to satisfy borrow checker
                            let target_x = target_mob.x;
                            let target_y = target_mob.y;

                            // After images
                            if swallow.after_image_countdown > 0 {
                                swallow.after_image_countdown -= 1;
                            } else {
                                create_swallow_after_image(
                                    self.core_state.entity_ids.next(),
                                    &swallow_mob,
                                    &swallow,
                                    &mut self.core_state.swallow_after_images,
                                );
                                swallow.after_image_countdown = AFTER_IMAGE_PERIOD;
                            }

                            // Check for collision

                            let distance_squared = (target_x - swallow_mob.x)
                                * (target_x - swallow_mob.x)
                                + (target_y - swallow_mob.y) * (target_y - swallow_mob.y);
                            let radius = STANDARD_ENEMY_RADIUS + SWALLOW_RADIUS;
                            if distance_squared < radius * radius {
                                // Apply a small impulse
                                if let Some(impulse) = self.core_state.impulses.get_mut(&target) {
                                    impulse.dx += 0.5 * swallow.rotation.cos();
                                    impulse.dy += 0.5 * swallow.rotation.sin();
                                }

                                // Deal damage
                                if let Some(health) = self.core_state.health.get_mut(&target) {
                                    health.curr_health -=
                                        self.config.common[SWALLOW_INDEX].base_damage;
                                }

                                // Alert the target
                                self.core_state.threats.insert(target, Threat {});

                                // Go back to curr_tower
                                if let Some(tower) = self.core_state.towers.get(&swallow.curr_tower)
                                {
                                    migrate_to_tower(
                                        swallow.curr_tower,
                                        tower,
                                        swallow,
                                        swallow_mob,
                                    );
                                }
                            }

                            // Aiming behavior: draw a line from the vanishing point
                            // to the target, and try to move along that line as the
                            // line moves. If the swallow cannot reach that line,
                            // fall back to chasing the target (classical pursuit).

                            // But first, if the target is close enough, we should aim
                            // directly at it. This check is redundant if speed is lower
                            // than the enemy size, which is probably always the case,
                            // but it's here just in case.

                            // We also aim for the target if we are facing away from it.
                            // This way we avoid getting stuck on the correct line
                            // moving in the wrong direction.

                            let dot = swallow.rotation.cos() * (target_x - swallow_mob.x)
                                + swallow.rotation.sin() * (target_y - swallow_mob.y);

                            let (goal_x, goal_y) =
                                if distance_squared < swallow.speed * swallow.speed || dot < 0.0 {
                                    // Go directly to the target if we are close enough
                                    (target_x, target_y)
                                } else if let Some(((x1, y1), (x2, y2))) = circle_line_intersection(
                                    swallow_mob.x,
                                    swallow_mob.y,
                                    swallow.speed,
                                    swallow.vanishing_x,
                                    swallow.vanishing_y,
                                    target_x,
                                    target_y,
                                ) {
                                    // We want the closer of the two options (closer
                                    // to the target)
                                    let dx1 = x1 - target_x;
                                    let dy1 = y1 - target_y;
                                    let dx2 = x2 - target_x;
                                    let dy2 = y2 - target_y;
                                    if dx1 * dx1 + dy1 * dy1 < dx2 * dx2 + dy2 * dy2 {
                                        (x1, y1)
                                    } else {
                                        (x2, y2)
                                    }
                                } else {
                                    // If we can't stay on the ideal line, set the current
                                    // position as the new vanishing point.
                                    swallow.vanishing_x = swallow_mob.x;
                                    swallow.vanishing_y = swallow_mob.y;
                                    (target_x, target_y)
                                };

                            // Now that we settled on a goal, we use the same logic as a
                            // missile would (but without acceleration)
                            let rotation =
                                f32::atan2(goal_y - swallow_mob.y, goal_x - swallow_mob.x);

                            ease_to_x_geometric(
                                &mut swallow.rotation,
                                &mut swallow.rotation_speed,
                                rotation,
                                0.0,
                                swallow.max_turn_speed,
                                swallow.rotation_accel,
                                crate::ease::Domain::Radian { miss_adjust: 0.9 },
                            );

                            // We have to access the hashmap again to get a
                            // mutable reference.
                            if let Some(swallow_mob) = self.core_state.mobs.get_mut(&entity) {
                                swallow_mob.x += swallow.speed * swallow.rotation.cos();
                                swallow_mob.y += swallow.speed * swallow.rotation.sin();
                            }
                        } else {
                            // If we can't find the enemy's info, it has already died.
                            // We intentionally don't change the swallow's rotation here,
                            // since doing so would look weird.
                            swallow.target = Target::Tower {
                                dist_from_tower_squared: f32::INFINITY,
                                net_rotation: 0.0,
                            }
                        }
                    }
                    Target::Tower {
                        dist_from_tower_squared,
                        net_rotation,
                    } => {
                        // After images
                        if swallow.after_image_countdown > 0 {
                            swallow.after_image_countdown -= 1;
                        } else {
                            create_swallow_after_image(
                                self.core_state.entity_ids.next(),
                                &swallow_mob,
                                &swallow,
                                &mut self.core_state.swallow_after_images,
                            );
                            swallow.after_image_countdown = AFTER_IMAGE_PERIOD;
                        }

                        if let Some(tower) = self.core_state.towers.get(&swallow.curr_tower) {
                            let (x, y) = tile_center(tower.row, tower.col);

                            let dx = x - swallow_mob.x;
                            let dy = y - swallow_mob.y;
                            let distance_squared = dx * dx + dy * dy;

                            // Avoid orbiting around towers.
                            // We decrease max rotation down to 25% once a
                            // swallow has rotated 360 degrees in search for
                            // a tower.
                            let extra_rotation = (net_rotation.abs() - TAU).max(0.0);
                            let max_turn_speed = if extra_rotation == 0.0 {
                                swallow.max_turn_speed
                            } else if extra_rotation < TAU {
                                swallow.max_turn_speed
                                    * (0.25 + 0.75 * (TAU - extra_rotation) / TAU)
                            } else {
                                swallow.max_turn_speed * 0.25
                            };

                            let old_rotation = swallow.rotation;
                            ease_to_x_geometric(
                                &mut swallow.rotation,
                                &mut swallow.rotation_speed,
                                f32::atan2(dy, dx),
                                0.0,
                                max_turn_speed,
                                swallow.rotation_accel,
                                crate::ease::Domain::Radian { miss_adjust: 0.9 },
                            );

                            // Keep distance and net rotation updated
                            swallow.target = Target::Tower {
                                dist_from_tower_squared: distance_squared,
                                // Take the modulo because swallow.rotation - old_rotation
                                // gets large when crossing between PI and -PI.
                                net_rotation: net_rotation
                                    + ((swallow.rotation - old_rotation) % PI),
                            };

                            // We have to access the hashmap again to get a
                            // mutable reference.
                            if let Some(swallow_mob) = self.core_state.mobs.get_mut(&entity) {
                                swallow_mob.x += swallow.speed * swallow.rotation.cos();
                                swallow_mob.y += swallow.speed * swallow.rotation.sin();

                                // Check for collision here so that we don't render any overshoot.

                                // We count a collision as a locally minimal
                                // distance within a tile. That way we can be
                                // generous with the collision area while still
                                // being relatively precise with the visual
                                // moment of collision.

                                let dx = x - swallow_mob.x;
                                let dy = y - swallow_mob.y;

                                let distance_squared = dx * dx + dy * dy;
                                if distance_squared < f32::TILE_SIZE * f32::TILE_SIZE / 4.0
                                    && distance_squared > dist_from_tower_squared
                                {
                                    swallow_mob.x = x;
                                    swallow_mob.y = y;
                                    swallow.target = Target::None;
                                }
                            }
                        }
                    }
                };
            }
        }
    }

    pub fn fade_swallow_after_images(&mut self) {
        let mut trash = Vec::new();
        for (&entity, after_image) in &mut self.core_state.swallow_after_images {
            after_image.age += 1;
            if after_image.age >= AFTER_IMAGE_DURATION {
                trash.push(entity);
            }
        }
        for entity in trash {
            self.core_state.swallow_after_images.remove(&entity);
        }
    }

    pub fn swallow_tower_targeting(&mut self) {
        for (entity, targeter) in &mut self.core_state.swallow_targeters {
            if let Some(tower) = self.core_state.towers.get(entity) {
                // No enemies can be seen if the tower isn't operational
                if tower.status != TowerStatus::Operational {
                    targeter.closest_distance_squared = f32::INFINITY;
                    continue;
                }

                let (x, y) = tile_center(tower.row, tower.col);
                if let Some((_mob_entity, closest_walker, distance_squared)) =
                    closest_walker(&self.core_state.walkers, &self.core_state.mobs, x, y)
                {
                    targeter.closest_x = closest_walker.x;
                    targeter.closest_y = closest_walker.y;
                    targeter.closest_distance_squared = distance_squared;
                } else {
                    targeter.closest_distance_squared = f32::INFINITY;
                }
            }
        }
    }
}
