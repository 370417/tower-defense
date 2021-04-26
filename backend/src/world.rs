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
    falcon::{create_falcon_tower, Falcon, TargetIndicator},
    graphics::{recycle_range, render_range, BuildProgressData, SpriteData},
    health::Health,
    map::{
        distances::{generate_dist_from_entrance, generate_dist_from_exit, Distances},
        entrances, parse, render_map, tile_center, Constants, Tile, MAP_0, MAP_WIDTH,
    },
    missile::{create_missile_tower, Missile, MissileSpawner},
    mob::Mob,
    pusillanimous::Pusillanimous,
    smoke::SmokeTrail,
    swallow::{create_swallow_tower, Swallow, SwallowAfterImage, SwallowTargeter},
    targeting::Threat,
    tower::{build_towers_by_pos, Tower, TowerStatus},
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

#[derive(Serialize, Deserialize, Default)]
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

#[derive(Default)]
pub struct RenderState {
    pub build_progress: BuildProgressData,
    pub preview_tower: Option<Tower>,
    pub smoke_trails: Map<u32, SmokeTrail>,
    pub sprite_data: SpriteData,
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
        }
    }

    pub fn update(&mut self) {
        match self.run_state {
            RunState::Paused | RunState::AutoPaused => return,
            RunState::Playing => {}
        }

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

    pub fn save(&mut self) {
        let bytes = match bincode::serialize(&self.core_state) {
            Ok(bytes) => bytes,
            Err(error) => {
                crate::log(&format!("Unable to save: {}", error));
                return;
            }
        };
        crate::log(&format!("size {}", bytes.len()));
    }

    // 2 types of user inputs:
    // 1. stuff that affects the game and needs to be shared over the network
    // 2. stuff that just affects the ui, like this hover_map
    pub fn hover_map(&self, _player: u32, row: usize, col: usize) {
        for tower in self.core_state.towers.values() {
            if (row, col) == (tower.row, tower.col) {
                let (x, y) = tile_center(row, col);
                render_range(x, y, tower.range);
                return;
            }
        }
        recycle_range();
    }

    pub fn run_state(&self) -> u8 {
        self.run_state as u8
    }

    // pub fn play/pause?
}

/// Stores the next available entity id (old ids are not reused)
#[derive(Serialize, Deserialize, Default)]
pub struct EntityIds(u32);

impl EntityIds {
    pub fn next(&mut self) -> u32 {
        let id = self.0;
        self.0 += 1;
        id
    }
}
