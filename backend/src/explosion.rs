//! A module for explosions and the impulses they generate.

use std::f32::consts::PI;

use crate::{
    collision::resolve_collisions,
    distance::fast_distance,
    graphics::{create_explosion, recycle_explosion, render_explosion},
    map::Constants,
    walker::STANDARD_ENEMY_RADIUS,
    world::{Map, World},
};

const EXPLOSION_DURATION: u32 = 16;

/// A stationary explosion that expands over time (eased by a sine wave).
/// We keep track of old_radius to tell when entities cross the edge of the
/// explosion.
pub struct Explosion {
    center_x: f32,
    center_y: f32,
    age: u32,
    old_radius: f32,
    radius: f32,
    max_radius: f32,
}

pub fn spawn_explosion(
    id: u32,
    explosions: &mut Map<u32, Explosion>,
    x: f32,
    y: f32,
    max_radius: f32,
) {
    create_explosion(id, x, y);

    explosions.insert(
        id,
        Explosion {
            center_x: x,
            center_y: y,
            age: 0,
            old_radius: 0.0,
            radius: 0.0,
            max_radius,
        },
    );
}

impl World {
    pub fn update_explosions(&mut self) {
        let mut trash = Vec::new();

        for (&entity, explosion) in &mut self.explosions {
            if explosion.age < EXPLOSION_DURATION {
                explosion.age += 1;
                let progress = explosion.age as f32 / EXPLOSION_DURATION as f32;
                let sin_eased_progress = (progress * PI / 2.0).sin();
                let cos_eased_progress = (progress * PI / 2.0).cos();
                explosion.old_radius = explosion.radius;
                explosion.radius = explosion.max_radius * sin_eased_progress;

                render_explosion(entity, explosion.radius, cos_eased_progress);

                // Apply impulses to mobs at the edge of the explosion.
                // We don't want to reapply impulses to mobs that have been hit
                // in the past. So we don't apply an impulse if the mob was
                // already touching the explosion the previous tick.

                for (entity, impulse) in &mut self.impulses {
                    if let Some(mob) = &mut self.mobs.get(entity) {
                        let distance_x = mob.x - explosion.center_x;
                        let distance_y = mob.y - explosion.center_y;
                        let distance_squared = distance_x * distance_x + distance_y * distance_y;

                        let old_distance_x = mob.old_x - explosion.center_x;
                        let old_distance_y = mob.old_y - explosion.center_y;
                        let old_distance_squared =
                            old_distance_x * old_distance_x + old_distance_y * old_distance_y;

                        let radius_squared = (STANDARD_ENEMY_RADIUS + explosion.radius)
                            * (STANDARD_ENEMY_RADIUS + explosion.radius);

                        let old_radius_squared = (STANDARD_ENEMY_RADIUS + explosion.old_radius)
                            * (STANDARD_ENEMY_RADIUS + explosion.old_radius);

                        if distance_squared <= radius_squared
                            && (explosion.old_radius == 0.0
                                || old_distance_squared > old_radius_squared)
                        {
                            let distance = distance_squared.sqrt();

                            let normalized_x = (mob.x - explosion.center_x) / distance;
                            let normalized_y = (mob.y - explosion.center_y) / distance;

                            impulse.dx += normalized_x
                                * (1.5 - 0.5 * explosion.radius / explosion.max_radius);
                            impulse.dy += normalized_y
                                * (1.5 - 0.5 * explosion.radius / explosion.max_radius);

                            // Make sure impulses don't get too big
                            let impulse_magnitude = fast_distance(impulse.dx, impulse.dy);
                            let max_magnitude = f32::TILE_SIZE / 2.0;
                            if impulse_magnitude > max_magnitude {
                                impulse.dx *= max_magnitude / impulse_magnitude;
                                impulse.dy *= max_magnitude / impulse_magnitude;
                            }
                        }
                    }
                }
            } else {
                trash.push(entity);
            }
        }
        for entity in trash {
            self.explosions.remove(&entity);
            recycle_explosion(entity);
        }
    }
}

/// Represents decaying impulses on an entity. Decay happens multiplicatively.
pub struct Impulse {
    pub dx: f32,
    pub dy: f32,
}

const IMPULSE_DECAY: f32 = 0.95;

impl World {
    pub fn update_impulses(&mut self) {
        for (entity, impulse) in &mut self.impulses {
            let mob = self.mobs.get_mut(entity);
            let is_walker = self.walkers.contains_key(entity);

            if let Some(mob) = mob {
                if is_walker {
                    let original_x = mob.x;
                    let original_y = mob.y;

                    mob.x += impulse.dx;
                    mob.y += impulse.dy;

                    let impact = resolve_collisions(
                        &self.map,
                        &mut mob.x,
                        &mut mob.y,
                        STANDARD_ENEMY_RADIUS,
                    );

                    // Update the impulse based on dx,dy post-collision.
                    // This way, collisions actually absorb momentum.
                    // Without this step, an entity could get pushed into a wall
                    // and maintain their original momentum after slipping off
                    // the wall.

                    // Add in the impact to make entities bounce off walls.

                    impulse.dx = mob.x - original_x + 0.5 * impact.0;
                    impulse.dy = mob.y - original_y + 0.5 * impact.1;
                }
            }

            impulse.dx *= IMPULSE_DECAY;
            impulse.dy *= IMPULSE_DECAY;
        }
    }
}
