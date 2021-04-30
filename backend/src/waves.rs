use std::{
    cmp::{Ordering, Reverse},
    collections::BinaryHeap,
};

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    explosion::Impulse,
    health::Health,
    map::true_tile_center,
    mob::Mob,
    walker::Walker,
    world::{CoreState, Map, World},
};

const TICKS_PER_SECOND: u32 = 60;
const SECONDS_PER_WAVE: u32 = 20;
const TICKS_PER_WAVE: u32 = TICKS_PER_SECOND * SECONDS_PER_WAVE;
const TICKS_BETWEEN_SPAWNS: u32 = 21;
const TICKS_TILL_FIRST_WAVE: u32 = 180;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct WaveSpawner {
    pub entrances: Vec<(usize, usize)>,
    ticks_till_next_wave: u32,
    next_wave_index: usize,
    queued_enemies: BinaryHeap<Reverse<QueuedEnemy>>,
}

impl WaveSpawner {
    pub fn new(entrances: Vec<(usize, usize)>) -> WaveSpawner {
        WaveSpawner {
            entrances,
            ticks_till_next_wave: TICKS_TILL_FIRST_WAVE,
            next_wave_index: 0,
            queued_enemies: BinaryHeap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Clone)]
struct QueuedEnemy {
    true_row: usize,
    true_col: usize,
    spawn_tick: u32,
    enemy_type: Enemy,
}

impl Ord for QueuedEnemy {
    fn cmp(&self, other: &Self) -> Ordering {
        self.spawn_tick.cmp(&other.spawn_tick)
    }
}

impl PartialOrd for QueuedEnemy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
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
    pub fn save_if_wave_start(&mut self) {
        if self.core_state.wave_spawner.ticks_till_next_wave == 0 {
            if self
                .config
                .waves
                .get(self.core_state.wave_spawner.next_wave_index)
                .is_some()
            {
                self.save();
            }
        }
    }

    pub fn spawn_mobs(&mut self) {
        // Queue waves
        if self.core_state.wave_spawner.ticks_till_next_wave == 0 {
            if let Some(wave) = self
                .config
                .waves
                .get(self.core_state.wave_spawner.next_wave_index)
            {
                self.core_state.wave_spawner.ticks_till_next_wave = TICKS_PER_WAVE - 1;
                self.core_state.wave_spawner.next_wave_index += 1;
                queue_wave(&mut self.core_state, wave);
            }
        } else {
            self.core_state.wave_spawner.ticks_till_next_wave -= 1;
        }

        // Spawn queued mobs
        while let Some(Reverse(queued_enemy)) = self.core_state.wave_spawner.queued_enemies.peek() {
            if queued_enemy.spawn_tick == self.core_state.tick {
                spawn_enemy(
                    self.core_state.entity_ids.next(),
                    queued_enemy.true_row,
                    queued_enemy.true_col,
                    &mut self.core_state.mobs,
                    &mut self.core_state.walkers,
                    &mut self.core_state.impulses,
                    &mut self.core_state.health,
                );
                self.core_state.wave_spawner.queued_enemies.pop();
            } else {
                break;
            }
        }
    }
}

#[wasm_bindgen]
impl World {
    /// Index of the next wave shown on the game's sidebar.
    /// If enemies are still spawning, this index will be the index of the
    /// still-spawning wave.
    /// If no enemies are spawning, this index will be the index of the next
    /// wave to spawn.
    /// If the last wave has finished spawning, return -1.
    pub fn next_wave_index(&self) -> i32 {
        let wave_spawner = &self.core_state.wave_spawner;
        if !wave_spawner.queued_enemies.is_empty() {
            wave_spawner.next_wave_index.saturating_sub(1) as i32
        } else if wave_spawner.next_wave_index < self.config.waves.len() {
            wave_spawner.next_wave_index as i32
        } else {
            -1
        }
    }

    /// Number of ticks until the next wave spawns, or 0 if a wave is currently
    /// spawning or no more waves remain.
    pub fn ticks_till_next_wave(&self) -> u32 {
        let wave_spawner = &self.core_state.wave_spawner;
        if !wave_spawner.queued_enemies.is_empty() {
            0
        } else if wave_spawner.next_wave_index < self.config.waves.len() {
            wave_spawner.ticks_till_next_wave
        } else {
            0
        }
    }

    pub fn send_next_wave(&mut self) {
        self.core_state.wave_spawner.ticks_till_next_wave = 0;
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
            core_state
                .wave_spawner
                .queued_enemies
                .push(Reverse(QueuedEnemy {
                    true_row,
                    true_col,
                    spawn_tick: core_state.tick + tick_delay,
                    enemy_type: group.r#type,
                }));
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
    health: &mut Map<u32, Health>,
) {
    let id = entity;
    let (x, y) = true_tile_center(true_row, true_col);
    mobs.insert(id, Mob::new(x, y));
    walkers.insert(id, Walker { speed: 1.5 });
    impulses.insert(id, Default::default());
    health.insert(entity, Health::new(100.0));
}
