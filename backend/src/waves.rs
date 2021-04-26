use serde::{Deserialize, Serialize};

use crate::{
    explosion::Impulse,
    map::true_tile_center,
    mob::Mob,
    walker::Walker,
    world::{CoreState, Map, World},
};

const TICKS_PER_SECOND: u32 = 60;
const SECONDS_PER_WAVE: u32 = 20;
const TICKS_PER_WAVE: u32 = TICKS_PER_SECOND * SECONDS_PER_WAVE;
const TICKS_BETWEEN_SPAWNS: u32 = 40;

#[derive(Serialize, Deserialize, Default)]
pub struct WaveSpawner {
    pub entrances: Vec<(usize, usize)>,
    queued_enemies: Vec<QueuedEnemy>,
}

impl WaveSpawner {
    pub fn new(entrances: Vec<(usize, usize)>) -> WaveSpawner {
        WaveSpawner {
            entrances,
            queued_enemies: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
struct QueuedEnemy {
    true_row: usize,
    true_col: usize,
    spawn_tick: u32,
    enemy_type: Enemy,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Wave {
    group: Vec<Group>,
}

#[derive(Serialize, Deserialize, Default)]
struct Group {
    size: u32,
    r#type: Enemy,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Enemy {
    Circle,
    Triangle,
    Square,
}

impl Default for Enemy {
    fn default() -> Self {
        Enemy::Circle
    }
}

impl World {
    pub fn spawn_mobs(&mut self) {
        // Queue waves
        if self.core_state.tick % TICKS_PER_WAVE == 0 {
            let i = self.core_state.tick / TICKS_PER_WAVE;
            if let Some(wave) = self.config.waves.get(i as usize) {
                queue_wave(&mut self.core_state, wave);
            }
        }

        // Spawn queued mobs
        for queued_enemy in &self.core_state.wave_spawner.queued_enemies {
            if queued_enemy.spawn_tick == self.core_state.tick {
                spawn_enemy(
                    self.core_state.entity_ids.next(),
                    queued_enemy.true_row,
                    queued_enemy.true_col,
                    &mut self.core_state.mobs,
                    &mut self.core_state.walkers,
                    &mut self.core_state.impulses,
                );
            }
        }
    }
}

fn queue_wave(core_state: &mut CoreState, wave: &Wave) {
    let mut i = 0;

    for group in &wave.group {
        for _ in 0..group.size {
            let entrance_i = i % core_state.wave_spawner.entrances.len();
            let tick_delay =
                (i / core_state.wave_spawner.entrances.len()) as u32 * TICKS_BETWEEN_SPAWNS;
            i += 1;

            let (true_row, true_col) = core_state.wave_spawner.entrances[entrance_i];
            core_state.wave_spawner.queued_enemies.push(QueuedEnemy {
                true_row,
                true_col,
                spawn_tick: core_state.tick + tick_delay,
                enemy_type: group.r#type,
            });
        }
    }
}

pub fn spawn_enemy(
    entity: u32,
    true_row: usize,
    true_col: usize,
    mobs: &mut Map<u32, Mob>,
    walkers: &mut Map<u32, Walker>,
    impulses: &mut Map<u32, Impulse>,
) {
    let id = entity;
    let (x, y) = true_tile_center(true_row, true_col);
    mobs.insert(id, Mob::new(x, y));
    walkers.insert(id, Walker { speed: 1.5 });
    impulses.insert(id, Default::default());
}
