//! Swallows and swallow towers
//!
//! Unlike missile towers, swallow towers take no part in targeting or spawning
//! projectiles, since the swallow is a persistent projectile.

use std::f32::consts::PI;

use crate::{
    collision::circle_line_intersection,
    ease::ease_to_x_geometric,
    graphics::{create_tower, SpriteData, SWALLOW_ID},
    map::{tile_center, Constants},
    mob::Mob,
    targeting::{find_target, Targeting},
    tower::{Range, Tower},
    walker::STANDARD_ENEMY_RADIUS,
    world::{EntityIds, Map, World},
};

const SPEED: f32 = 5.2;
const ROTATION_ACCEL: f32 = 0.05;
const MAX_TURN_SPEED: f32 = 0.25;
const SWALLOW_RADIUS: f32 = f32::TILE_SIZE * 0.3;
const RANGE: f32 = 2.6 * f32::TILE_SIZE;

const AFTER_IMAGE_PERIOD: u32 = 2;
const AFTER_IMAGE_DURATION: u32 = 17;

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

pub struct SwallowAfterImage {
    x: f32,
    y: f32,
    rotation: f32,
    age: u32,
}

/// Component for tower entities. Keeps track of the closest walker and only
/// updates when necessary. This way, if 100 swallows need to scan 100 towers
/// to see if they have an enemy in range, and all 100 swallows have different
/// ranges, we can avoid looping over all enemies over and over.
pub struct SwallowTargeter {
    pub closest_distance_squared: f32,
    pub closest_x: f32,
    pub closest_y: f32,
}

pub enum Target {
    Enemy(u32),
    Tower { dist_from_tower_squared: f32 },
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

    // pub fn render(&self, id: u32, frame_fudge: f32, world: &World) {
    //     if let Some(mob) = world.mobs.get(&id) {
    //         let x = mob.x + frame_fudge * (mob.x - mob.old_x);
    //         let y = mob.y + frame_fudge * (mob.y - mob.old_y);
    //         render_swallow(id, x, y, self.rotation, 0.0);
    //     }
    // }

    pub fn dump(&self, id: &u32, data: &mut SpriteData, mobs: &Map<u32, Mob>, frame_fudge: f32) {
        if let Some(mob) = mobs.get(id) {
            data.push(
                SWALLOW_ID,
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
            SWALLOW_ID,
            self.x,
            self.y,
            self.rotation,
            0.2 - 0.2 * (self.age as f32 + frame_fudge) / AFTER_IMAGE_DURATION as f32,
            0x000000,
        );
    }

    // pub fn render(&self, id: u32, frame_fudge: f32) {
    //     render_swallow(
    //         id,
    //         self.x,
    //         self.y,
    //         self.rotation,
    //         0.8 + 0.2 * (self.age as f32 + frame_fudge) / AFTER_IMAGE_DURATION as f32,
    //     );
    // }
}

pub fn create_swallow_tower(
    entities: &mut EntityIds,
    row: usize,
    col: usize,
    towers: &mut Map<u32, Tower>,
    swallows: &mut Map<u32, Swallow>,
    mobs: &mut Map<u32, Mob>,
) {
    let tower_entity = entities.next();
    let swallow_entity = entities.next();

    let (x, y) = tile_center(row, col);

    towers.insert(
        tower_entity,
        Tower {
            row,
            col,
            range: Range::Circle { radius: RANGE },
        },
    );
    swallows.insert(swallow_entity, Swallow::new(-PI / 2.0, x, y, tower_entity));
    mobs.insert(swallow_entity, Mob::new(x, y));

    create_tower(tower_entity, row, col);
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

impl World {
    pub fn fly_swallows(&mut self) {
        for (&entity, swallow) in &mut self.swallows {
            if let Some(swallow_mob) = self.mobs.get(&entity) {
                match swallow.target {
                    Target::None => {
                        // once we implement migration, we'll want to check
                        // first if we should return home

                        swallow.after_image_countdown =
                            swallow.after_image_countdown.saturating_sub(1);

                        if let Some(tower) = self.towers.get(&swallow.curr_tower) {
                            if let Some((target, target_x, target_y)) = find_target(
                                swallow_mob.x,
                                swallow_mob.y,
                                tower.range,
                                Targeting::Close,
                                &self.walkers,
                                &self.mobs,
                                &self.map,
                                &self.dist_from_entrance,
                                &self.dist_from_exit,
                            ) {
                                swallow.target = Target::Enemy(target);
                                swallow.rotation =
                                    f32::atan2(target_y - swallow_mob.y, target_x - swallow_mob.x);
                                swallow.vanishing_x = swallow_mob.x;
                                swallow.vanishing_y = swallow_mob.y;
                            }
                        }
                    }
                    Target::Enemy(target) => {
                        if let Some(target_mob) = self.mobs.get(&target) {
                            // Copy target's coordinates to satisfy borrow checker
                            let target_x = target_mob.x;
                            let target_y = target_mob.y;

                            // After images
                            if swallow.after_image_countdown > 0 {
                                swallow.after_image_countdown -= 1;
                            } else {
                                create_swallow_after_image(
                                    self.entity_ids.next(),
                                    &swallow_mob,
                                    &swallow,
                                    &mut self.swallow_after_images,
                                );
                                swallow.after_image_countdown = AFTER_IMAGE_PERIOD;
                            }

                            // Check for collision

                            let distance_squared = (target_x - swallow_mob.x)
                                * (target_x - swallow_mob.x)
                                + (target_y - swallow_mob.y) * (target_y - swallow_mob.y);
                            let radius = STANDARD_ENEMY_RADIUS + SWALLOW_RADIUS;
                            if distance_squared < radius * radius {
                                swallow.target = Target::Tower {
                                    dist_from_tower_squared: f32::INFINITY,
                                };
                                // Apply a small impulse
                                if let Some(impulse) = self.impulses.get_mut(&target) {
                                    impulse.dx += 0.5 * swallow.rotation.cos();
                                    impulse.dy += 0.5 * swallow.rotation.sin();
                                }

                                // Point the swallow back towards its tower.
                                if let Some(tower) = self.towers.get(&swallow.curr_tower) {
                                    let (x, y) = tile_center(tower.row, tower.col);
                                    swallow.rotation =
                                        f32::atan2(y - swallow_mob.y, x - swallow_mob.x);
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
                                swallow.max_turn_speed,
                                swallow.rotation_accel,
                                crate::ease::Domain::Radian { miss_adjust: 0.9 },
                            );

                            // We have to access the hashmap again to get a
                            // mutable reference.
                            if let Some(swallow_mob) = self.mobs.get_mut(&entity) {
                                swallow_mob.x += swallow.speed * swallow.rotation.cos();
                                swallow_mob.y += swallow.speed * swallow.rotation.sin();
                            }
                        }
                    }
                    Target::Tower {
                        dist_from_tower_squared,
                    } => {
                        // After images
                        if swallow.after_image_countdown > 0 {
                            swallow.after_image_countdown -= 1;
                        } else {
                            create_swallow_after_image(
                                self.entity_ids.next(),
                                &swallow_mob,
                                &swallow,
                                &mut self.swallow_after_images,
                            );
                            swallow.after_image_countdown = AFTER_IMAGE_PERIOD;
                        }

                        if let Some(tower) = self.towers.get(&swallow.curr_tower) {
                            let (x, y) = tile_center(tower.row, tower.col);

                            let dx = x - swallow_mob.x;
                            let dy = y - swallow_mob.y;
                            let distance_squared = dx * dx + dy * dy;

                            // // Check for collision with the tower.
                            // // We count a collision as a locally minimal
                            // // distance within a tile. That way we can be
                            // // generous with the collision area while still
                            // // being relatively precise with the visual
                            // // moment of collision.

                            // if distance_squared < f32::TILE_SIZE * f32::TILE_SIZE / 4.0
                            //     && distance_squared > dist_from_tower_squared
                            // {
                            //     if let Some(swallow_mob) = self.mobs.get_mut(&entity) {
                            //         swallow_mob.x = x;
                            //         swallow_mob.y = y;
                            //     }
                            //     swallow.target = Target::None;
                            //     continue;
                            // }
                            swallow.target = Target::Tower {
                                dist_from_tower_squared: distance_squared,
                            };

                            ease_to_x_geometric(
                                &mut swallow.rotation,
                                &mut swallow.rotation_speed,
                                f32::atan2(dy, dx),
                                swallow.max_turn_speed,
                                swallow.rotation_accel,
                                crate::ease::Domain::Radian { miss_adjust: 0.9 },
                            );

                            // We have to access the hashmap again to get a
                            // mutable reference.
                            if let Some(swallow_mob) = self.mobs.get_mut(&entity) {
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
        for (&entity, after_image) in &mut self.swallow_after_images {
            after_image.age += 1;
            if after_image.age >= AFTER_IMAGE_DURATION {
                trash.push(entity);
            }
        }
        for entity in trash {
            self.swallow_after_images.remove(&entity);
        }
    }
}
