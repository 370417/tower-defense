use std::collections::VecDeque;

use fnv::FnvBuildHasher;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    build::BuildOrder,
    config::Config,
    explosion::{Explosion, Impulse},
    factory::Factory,
    falcon::{Falcon, TargetIndicator},
    graphics::RenderState,
    health::Health,
    map::{
        distances::{generate_dist_from_entrance, generate_dist_from_exit, Distances},
        entrances, parse, render_map, Constants, Tile, MAP_0, MAP_WIDTH,
    },
    missile::{Missile, MissileSpawner},
    mob::Mob,
    pusillanimous::Pusillanimous,
    swallow::{Swallow, SwallowAfterImage, SwallowTargeter},
    targeting::Threat,
    tower::Tower,
    walker::Walker,
    waves::WaveSpawner,
};

/// A hash map.
///
/// World uses IndexMaps rather than HashMaps to guarantee that iteration
/// order is deterministic. HashMaps might end up being safe, especially in
/// WebAssembly, but IndexMaps are an easy safe bet.
///
/// We also use Fnv hashing instead of randomly seeded hashing, mainly for
/// performance, but also to eliminate another potential source of
/// nondeterminism.

pub type Map<K, V> = IndexMap<K, V, FnvBuildHasher>;

/// Stores global state as Arrays-of-Structures (actually, IndexMaps of
/// Structures).
///
/// At first, I was going to use an ECS library, but Rust ECS implementations
/// seem to be designed around multithreading, and I have a hard requirement
/// of determinism. In a single threaded context, those libraries might be
/// deterministic, but it feels safer to roll my own state.

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CoreState {
    pub tick: u32,
    pub build_queue: VecDeque<BuildOrder>,
    pub entity_ids: EntityIds,
    #[serde(with = "indexmap::serde_seq")]
    pub explosions: Map<u32, Explosion>,
    #[serde(with = "indexmap::serde_seq")]
    pub factories: Map<u32, Factory>,
    #[serde(with = "indexmap::serde_seq")]
    pub falcons: Map<u32, Falcon>,
    #[serde(with = "indexmap::serde_seq")]
    pub health: Map<u32, Health>,
    #[serde(with = "indexmap::serde_seq")]
    pub impulses: Map<u32, Impulse>,
    #[serde(with = "indexmap::serde_seq")]
    pub missile_spawners: Map<u32, MissileSpawner>,
    #[serde(with = "indexmap::serde_seq")]
    pub missiles: Map<u32, Missile>,
    #[serde(with = "indexmap::serde_seq")]
    pub mobs: Map<u32, Mob>,
    #[serde(with = "indexmap::serde_seq")]
    pub pusillanimous: Map<u32, Pusillanimous>,
    #[serde(with = "indexmap::serde_seq")]
    pub swallow_after_images: Map<u32, SwallowAfterImage>,
    #[serde(with = "indexmap::serde_seq")]
    pub swallow_targeters: Map<u32, SwallowTargeter>,
    #[serde(with = "indexmap::serde_seq")]
    pub swallows: Map<u32, Swallow>,
    #[serde(with = "indexmap::serde_seq")]
    pub target_indicators: Map<u32, TargetIndicator>,
    #[serde(with = "indexmap::serde_seq")]
    pub threats: Map<u32, Threat>,
    #[serde(with = "indexmap::serde_seq")]
    pub towers: Map<u32, Tower>,
    #[serde(with = "indexmap::serde_seq")]
    pub towers_by_pos: Map<(usize, usize), u32>,
    #[serde(with = "indexmap::serde_seq")]
    pub under_construction: Map<u32, ()>, // IndexSet can't be auto-serialized
    #[serde(with = "indexmap::serde_seq")]
    pub walkers: Map<u32, Walker>,
    pub wave_spawner: WaveSpawner,
}

pub struct LevelState {
    pub level_id: u32,
    pub dist_from_entrance: Distances,
    pub dist_from_exit: Distances,
    pub map: Vec<Tile>,
}

#[derive(Clone, Copy)]
pub enum RunState {
    Paused,
    AutoPaused,
    Playing,
}

#[wasm_bindgen]
pub struct World {
    #[wasm_bindgen(skip)]
    pub run_state: RunState,
    #[wasm_bindgen(skip)]
    pub config: Config,
    #[wasm_bindgen(skip)]
    pub core_state: CoreState,
    #[wasm_bindgen(skip)]
    pub level_state: LevelState,
    #[wasm_bindgen(skip)]
    pub render_state: RenderState,
    #[wasm_bindgen(skip)]
    pub saved_states: Vec<CoreState>,
}

#[wasm_bindgen]
impl World {
    pub fn new(config: &str) -> World {
        let config = match toml::from_str(config) {
            Ok(config) => config,
            Err(error) => {
                crate::log(&format!("{}", error));
                panic!("Unable to read config");
            }
        };

        let map = parse(&MAP_0);

        render_map(&map);

        // Avoid entity 0 because the fnv hash doesn't like 0s
        let entity_ids = EntityIds(1);

        // // any initial towers start off operational
        // for tower in towers.values_mut() {
        //     tower.status = TowerStatus::Operational;
        // }

        let wave_spawner = WaveSpawner::new(entrances(&map));

        World {
            run_state: RunState::AutoPaused,
            config,
            core_state: CoreState {
                tick: 0,
                entity_ids,
                wave_spawner,
                ..Default::default()
            },
            level_state: LevelState {
                level_id: 0,
                dist_from_entrance: generate_dist_from_entrance(&map),
                dist_from_exit: generate_dist_from_exit(&map),
                map,
            },
            render_state: Default::default(),
            saved_states: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        match self.run_state {
            RunState::Paused | RunState::AutoPaused => return,
            RunState::Playing => {}
        }

        // Never do anything before saving
        self.save_if_wave_start();

        self.remember_mob_positions();
        self.update_pusillanimity();
        self.walk();
        self.fly_missiles();
        self.swallow_tower_targeting();
        self.fly_swallows();
        self.fade_swallow_after_images();
        self.fly_falcons();
        self.update_impulses();
        self.update_explosions();
        self.operate_missile_towers();
        self.update_smoke();
        self.handle_dead();
        self.spawn_mobs();
        self.progress_build();
        for (entity, mob) in &mut self.core_state.mobs {
            if self.core_state.walkers.contains_key(entity)
                && mob.x >= (MAP_WIDTH * usize::TILE_SIZE + usize::TILE_SIZE / 2) as f32
            {
                mob.x = -(f32::TILE_SIZE / 2.0);
            }
        }

        self.core_state.tick += 1;
    }

    pub fn restore(&mut self) {
        let old_tick = self.core_state.tick;
        if let Some(saved) = self.saved_states.pop() {
            self.core_state = saved;
            // Avoid weird time travel & underflow with visuals
            self.render_state = Default::default();
            // Go back one more time if the restore was <= 3s.
            // If we don't do this, it is nigh impossible to restore more than
            // one save state back (since restoring immediately saves again)
            if old_tick - self.core_state.tick <= 3 * 60 {
                self.restore();
            }
        }
    }

    pub fn run_state(&self) -> u8 {
        self.run_state as u8
    }

    pub fn play_pause(&mut self) {
        self.run_state = match self.run_state {
            RunState::Paused | RunState::AutoPaused => RunState::Playing,
            RunState::Playing => RunState::Paused,
        };
    }
}

impl World {
    pub fn save(&mut self) {
        self.saved_states.push(self.core_state.clone());
    }
}

/// Stores the next available entity id (old ids are not reused)
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct EntityIds(u32);

impl EntityIds {
    pub fn next(&mut self) -> u32 {
        let id = self.0;
        self.0 += 1;
        id
    }
}
