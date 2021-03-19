//! Swallows and swallow towers

use std::f32::consts::{PI, TAU};

use crate::{
    collision::circle_line_intersection,
    graphics::{create_missile, recycle_missile, render_missile},
    map::Constants,
    missile::MissileTower,
    mob::Mob,
    walker::STANDARD_ENEMY_RADIUS,
    world::{Map, World},
};

const SPEED: f32 = 4.1;
const MAX_TURN_SPEED: f32 = 0.05;

pub struct Swallow {
    pub target: u32,
    pub rotation: f32,
    pub max_turn_speed: f32,
    pub speed: f32,
    pub vanishing_x: f32,
    pub vanishing_y: f32,
}

impl Swallow {
    fn new(target: u32, rotation: f32, tower_x: f32, tower_y: f32) -> Swallow {
        Swallow {
            target,
            rotation,
            max_turn_speed: MAX_TURN_SPEED,
            speed: SPEED,
            vanishing_x: tower_x,
            vanishing_y: tower_y,
        }
    }

    pub fn render(&self, id: u32, frame_fudge: f32, world: &World) {
        if let Some(mob) = world.mobs.get(&id) {
            let x = mob.x + frame_fudge * (mob.x - mob.old_x);
            let y = mob.y + frame_fudge * (mob.y - mob.old_y);
            render_missile(id, x, y, self.rotation);
        }
    }
}

pub fn spawn_swallow(
    entity: u32,
    target: u32,
    tower: &MissileTower,
    target_x: f32,
    target_y: f32,
    swallows: &mut Map<u32, Swallow>,
    mobs: &mut Map<u32, Mob>,
) {
    // Add one half because the top and left edges of tiles are occupied by
    // the border between tiles
    let tower_x = (tower.col * usize::TILE_SIZE + usize::TILE_SIZE / 2) as f32 + 0.5;
    let tower_y = (tower.row * usize::TILE_SIZE + usize::TILE_SIZE / 2) as f32 + 0.5;

    let rotation = (target_y - tower_y).atan2(target_x - tower_x);
    swallows.insert(entity, Swallow::new(target, rotation, tower_x, tower_y));
    mobs.insert(entity, Mob::new(tower_x, tower_y));
    create_missile(entity);
}

impl World {
    pub fn fly_swallows(&mut self) {
        let mut trash = Vec::new();
        for (&entity, swallow) in &mut self.swallows {
            if let Some(target_mob) = self.mobs.get(&swallow.target) {
                // Copy target's coordinates to satisfy borrow checker
                let target_x = target_mob.x;
                let target_y = target_mob.y;

                if let Some(swallow_mob) = self.mobs.get_mut(&entity) {
                    // Check for collision
                    // The tip of a swallow extends out in front if its
                    // x,y position.
                    // Later, the swallow will be a square centered on its
                    // position, but for now, we are using the missile graphic
                    // so we skip this step.

                    let distance_squared = (target_x - swallow_mob.x) * (target_x - swallow_mob.x)
                        + (target_y - swallow_mob.y) * (target_y - swallow_mob.y);
                    if distance_squared < STANDARD_ENEMY_RADIUS * STANDARD_ENEMY_RADIUS {
                        trash.push(entity);
                        continue;
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
                    let rotation = f32::atan2(goal_y - swallow_mob.y, goal_x - swallow_mob.x);
                    let turn_amount = (rotation - swallow.rotation) % TAU;

                    // Adjust turn_amount to range from -pi to +pi
                    let turn_amount = if turn_amount < -PI {
                        turn_amount + TAU
                    } else if turn_amount > PI {
                        turn_amount - TAU
                    } else {
                        turn_amount
                    };

                    let clamped_turn_speed = turn_amount
                        .max(-swallow.max_turn_speed)
                        .min(swallow.max_turn_speed);
                    swallow.rotation += clamped_turn_speed;

                    swallow_mob.x += swallow.speed * swallow.rotation.cos();
                    swallow_mob.y += swallow.speed * swallow.rotation.sin();
                }
            }
        }
        for entity in trash {
            self.swallows.remove(&entity);
            self.mobs.remove(&entity);
            recycle_missile(entity);
        }
    }
}
