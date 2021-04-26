//! A module for explosions and the impulses they generate.

use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

use crate::{
    collision::resolve_collisions,
    distance::fast_distance,
    graphics::{create_explosion, recycle_explosion, render_explosion},
    walker::{walk_direction, Velocity, STANDARD_ENEMY_RADIUS},
    world::{Map, World},
};

const EXPLOSION_DURATION: u32 = 16;

/// A stationary explosion that expands over time (eased by a sine wave).
/// We keep track of old_radius to tell when entities cross the edge of the
/// explosion.
#[derive(Serialize, Deserialize)]
pub struct Explosion {
    center_x: f32,
    center_y: f32,
    age: u32,
    old_radius: f32,
    radius: f32,
    max_radius: f32,
    touched_entities: Vec<u32>,
    damage: f32,
}

pub fn spawn_explosion(
    id: u32,
    explosions: &mut Map<u32, Explosion>,
    x: f32,
    y: f32,
    max_radius: f32,
    damage: f32,
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
            touched_entities: Vec::new(),
            damage,
        },
    );
}

impl World {
    pub fn update_explosions(&mut self) {
        let mut trash = Vec::new();

        for (&entity, explosion) in &mut self.core_state.explosions {
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

                for (entity, impulse) in &mut self.core_state.impulses {
                    if let Some(mob) = &mut self.core_state.mobs.get(entity) {
                        let distance_x = mob.x - explosion.center_x;
                        let distance_y = mob.y - explosion.center_y;
                        let distance_squared = distance_x * distance_x + distance_y * distance_y;

                        let radius_squared = (STANDARD_ENEMY_RADIUS + explosion.radius)
                            * (STANDARD_ENEMY_RADIUS + explosion.radius);

                        if distance_squared <= radius_squared
                            && !explosion.touched_entities.contains(entity)
                        {
                            explosion.touched_entities.push(*entity);
                            let distance = distance_squared.sqrt();

                            let normalized_x = (mob.x - explosion.center_x) / distance;
                            let normalized_y = (mob.y - explosion.center_y) / distance;

                            impulse.dx += normalized_x
                                * (1.5 - 0.5 * explosion.radius / explosion.max_radius);
                            impulse.dy += normalized_y
                                * (1.5 - 0.5 * explosion.radius / explosion.max_radius);

                            // Deal damage
                            if let Some(health) = self.core_state.health.get_mut(entity) {
                                health.curr_health -= explosion.damage;
                            }
                        }
                    }
                }
            } else {
                trash.push(entity);
            }
        }
        for entity in trash {
            self.core_state.explosions.remove(&entity);
            recycle_explosion(entity);
        }
    }
}

/// Represents decaying impulses on an entity. Decay happens multiplicatively.
#[derive(Serialize, Deserialize, Default)]
pub struct Impulse {
    pub dx: f32,
    pub dy: f32,
}

const IMPULSE_DECAY: f32 = 0.95;

impl World {
    pub fn update_impulses(&mut self) {
        for (entity, impulse) in &mut self.core_state.impulses {
            let mob = self.core_state.mobs.get_mut(entity);
            let is_walker = self.core_state.walkers.contains_key(entity);

            if let Some(mob) = mob {
                if is_walker {
                    let original_x = mob.x;
                    let original_y = mob.y;

                    // Make sure impulses don't get too big
                    let impulse_magnitude = fast_distance(impulse.dx, impulse.dy);
                    let max_magnitude = STANDARD_ENEMY_RADIUS / 2.5;
                    if impulse_magnitude > max_magnitude {
                        impulse.dx *= max_magnitude / impulse_magnitude;
                        impulse.dy *= max_magnitude / impulse_magnitude;
                    }

                    mob.x += impulse.dx;
                    mob.y += impulse.dy;

                    let impact = resolve_collisions(
                        &self.level_state.map,
                        &mut mob.x,
                        &mut mob.y,
                        STANDARD_ENEMY_RADIUS,
                    );

                    let zero = Velocity { dx: 0.0, dy: 0.0 };
                    if walk_direction(&self.level_state.map, mob.x, mob.y) == zero {
                        // potentially remove the mob?
                    }

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
