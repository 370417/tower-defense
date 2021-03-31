//! Missiles and missile towers.

use crate::{
    ease::ease_to_x_geometric,
    explosion::spawn_explosion,
    graphics::{create_missile, create_tower, recycle_missile, render_missile},
    map::{tile_center, Constants},
    mob::Mob,
    smoke::spawn_smoke_trail,
    targeting::{find_target, Targeting},
    tower::{Range, Tower},
    walker::STANDARD_ENEMY_RADIUS,
    world::{Map, World},
};

const MAX_SPEED: f32 = 5.0;
const MAX_TURN_SPEED: f32 = 0.13;
const ACCELERATION: f32 = 0.31;
const ROTATION_ACCEL: f32 = 0.05;

// pub struct MissileTower {
//     pub row: usize,
//     pub col: usize,
//     pub reload_countdown: u32,
//     pub reload_cost: u32,
// }

pub struct MissileSpawner {
    pub reload_countdown: u32,
    pub reload_cost: u32,
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
}

pub fn create_misile_tower(
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
            reload_countdown: 0,
            reload_cost: 60,
        },
    );

    create_tower(entity, row, col);
}

fn spawn_missile(
    entity: u32,
    target: u32,
    tower: &Tower,
    target_x: f32,
    target_y: f32,
    missiles: &mut Map<u32, Missile>,
    mobs: &mut Map<u32, Mob>,
) {
    let (tower_x, tower_y) = tile_center(tower.row, tower.col);

    let rotation = (target_y - tower_y).atan2(target_x - tower_x);
    missiles.insert(entity, Missile::new(target, rotation));
    mobs.insert(entity, Mob::new(tower_x, tower_y));
    create_missile(entity);
}

impl World {
    pub fn operate_missile_towers(&mut self) {
        for (entity, spawner) in &mut self.missile_spawners {
            if let Some(tower) = self.towers.get(entity) {
                if spawner.reload_countdown > 0 {
                    spawner.reload_countdown -= 1;
                } else {
                    let (tower_x, tower_y) = tile_center(tower.row, tower.col);
                    if let Some((target, target_x, target_y)) = find_target(
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
                        spawn_smoke_trail(&mut self.entity_ids, &mut self.smoke_trails, entity);
                        spawner.reload_countdown = spawner.reload_cost;
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

                    ease_to_x_geometric(
                        &mut missile.rotation,
                        &mut missile.rotation_speed,
                        rotation,
                        missile.max_turn_speed,
                        missile.rotation_acceleration,
                        crate::ease::Domain::Radian,
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
            recycle_missile(entity);
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
