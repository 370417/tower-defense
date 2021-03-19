//! Missiles and missile towers.

use std::f32::consts::{PI, TAU};

use crate::{
    explosion::spawn_explosion,
    graphics::{create_missile, recycle_missile, render_missile},
    map::Constants,
    mob::Mob,
    smoke::spawn_smoke_trail,
    swallow::spawn_swallow,
    walker::{Walker, STANDARD_ENEMY_RADIUS},
    world::{Map, World},
};

const MAX_SPEED: f32 = 5.0;
const MAX_TURN_SPEED: f32 = 0.15;
const ACCELERATION: f32 = 0.31;

pub struct MissileTower {
    pub row: usize,
    pub col: usize,
    pub reload_countdown: u32,
    pub reload_cost: u32,
}

fn spawn_missile(
    entity: u32,
    target: u32,
    tower: &MissileTower,
    target_x: f32,
    target_y: f32,
    missiles: &mut Map<u32, Missile>,
    mobs: &mut Map<u32, Mob>,
) {
    // Add one half because the top and left edges of tiles are occupied by
    // the border between tiles
    let tower_x = (tower.col * usize::TILE_SIZE + usize::TILE_SIZE / 2) as f32 + 0.5;
    let tower_y = (tower.row * usize::TILE_SIZE + usize::TILE_SIZE / 2) as f32 + 0.5;

    let rotation = (target_y - tower_y).atan2(target_x - tower_x);
    missiles.insert(entity, Missile::new(target, rotation));
    mobs.insert(entity, Mob::new(tower_x, tower_y));
    create_missile(entity);
}

impl World {
    pub fn operate_missile_towers(&mut self) {
        for tower in self.missile_towers.values_mut() {
            tower.reload_countdown = tower.reload_countdown.saturating_sub(1);
            if tower.reload_countdown == 0 {
                if let Some((target, target_x, target_y)) =
                    find_target(&self.walkers, &self.mobs, tower.row, tower.col)
                {
                    let entity = self.entity_ids.next();
                    spawn_missile(
                        entity,
                        target,
                        tower,
                        target_x,
                        target_y,
                        &mut self.missiles,
                        &mut self.mobs,
                    );
                    spawn_swallow(
                        self.entity_ids.next(),
                        target,
                        tower,
                        target_x,
                        target_y,
                        &mut self.swallows,
                        &mut self.mobs,
                    );
                    spawn_smoke_trail(&mut self.entity_ids, &mut self.smoke_trails, entity);
                    tower.reload_countdown = tower.reload_cost;
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
            if let Some(target_mob) = self.mobs.get(&missile.target) {
                // Copy target's coordinates to satisfy borrow checker
                let target_x = target_mob.x;
                let target_y = target_mob.y;

                if let Some(missile_mob) = self.mobs.get_mut(&entity) {
                    // Check for collision
                    // The tip of a missile extends out in front of its
                    // x,y position.
                    let missile_length = 10.0;
                    let target_radius = STANDARD_ENEMY_RADIUS;
                    let missile_tip_x = missile_mob.x + missile_length * missile.rotation.cos();
                    let missile_tip_y = missile_mob.y + missile_length * missile.rotation.sin();
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
                    }

                    // Aim toward the target
                    let rotation = f32::atan2(target_y - missile_mob.y, target_x - missile_mob.x);
                    let turn_amount = (rotation - missile.rotation) % TAU;

                    // Adjust turn_amount to range from -pi to +pi
                    let turn_amount = if turn_amount < -PI {
                        turn_amount + TAU
                    } else if turn_amount > PI {
                        turn_amount - TAU
                    } else {
                        turn_amount
                    };

                    // Reduce turn speed when moving fast
                    let max_turn_speed = missile.max_turn_speed * missile.max_speed
                        / (missile.max_speed + missile.speed);

                    let clamped_turn_speed = turn_amount.max(-max_turn_speed).min(max_turn_speed);
                    missile.rotation += clamped_turn_speed;

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
            recycle_missile(entity);
        }
    }
}

fn find_target(
    walkers: &Map<u32, Walker>,
    mobs: &Map<u32, Mob>,
    row: usize,
    col: usize,
) -> Option<(u32, f32, f32)> {
    let tower_x = (col * usize::TILE_SIZE + usize::TILE_SIZE / 2) as f32 + 0.5;
    let tower_y = (row * usize::TILE_SIZE + usize::TILE_SIZE / 2) as f32 + 0.5;

    let mut first_walker = None;
    let mut max_progress = 0.0;

    let range = 300.0;

    for (entity, walker) in walkers {
        if let Some(mob) = mobs.get(entity) {
            let dx = mob.x - tower_x;
            let dy = mob.y - tower_y;
            let distance_squared = dx * dx + dy * dy;
            if walker.progress > max_progress && distance_squared <= range * range {
                max_progress = walker.progress;
                first_walker = Some((*entity, mob.x, mob.y));
            }
        }
    }

    first_walker
}

pub struct Missile {
    pub target: u32,
    pub rotation: f32,
    pub max_turn_speed: f32,
    pub speed: f32,
    pub max_speed: f32,
    pub acceleration: f32,
}

impl Missile {
    fn new(target: u32, rotation: f32) -> Missile {
        Missile {
            target,
            rotation,
            max_turn_speed: MAX_TURN_SPEED,
            max_speed: MAX_SPEED,
            speed: 0.0,
            acceleration: ACCELERATION,
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
