use std::collections::VecDeque;

use fnv::FnvBuildHasher;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    build::BuildOrder,
    explosion::{Explosion, Impulse},
    factory::Factory,
    falcon::{create_falcon_tower, Falcon, TargetIndicator},
    graphics::{recycle_range, render_range, BuildProgressData, SpriteData},
    map::{
        distances::{generate_dist_from_entrance, generate_dist_from_exit, Distances},
        parse, render_map, tile_center, Constants, Tile, MAP_0, MAP_WIDTH,
    },
    missile::{create_missile_tower, Missile, MissileSpawner},
    mob::Mob,
    pusillanimous::Pusillanimous,
    smoke::SmokeTrail,
    swallow::{create_swallow_tower, Swallow, SwallowAfterImage, SwallowTargeter},
    targeting::Threat,
    tower::{build_towers_by_pos, Tower, TowerStatus},
    walker::Walker,
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
    pub smoke_trails: Map<u32, SmokeTrail>,
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
}

#[derive(Default)]
pub struct RenderState {
    pub preview_tower: Option<Tower>,
    pub sprite_data: SpriteData,
    pub build_progress: BuildProgressData,
}

pub struct LevelState {
    pub level_id: u32,
    pub dist_from_entrance: Distances,
    pub dist_from_exit: Distances,
    pub map: Vec<Tile>,
}

#[wasm_bindgen]
pub struct World {
    pub game_speed: u32,
    #[wasm_bindgen(skip)]
    pub core_state: CoreState,
    #[wasm_bindgen(skip)]
    pub level_state: LevelState,
    #[wasm_bindgen(skip)]
    pub render_state: RenderState,
}

#[wasm_bindgen]
impl World {
    pub fn new() -> World {
        let map = parse(&MAP_0);

        render_map(&map);

        // Avoid entity 0 because the fnv hash doesn't like 0s
        let mut entity_ids = EntityIds(1);

        let mut mobs = IndexMap::with_hasher(Default::default());
        let mut walkers = IndexMap::with_hasher(Default::default());
        let mut impulses = IndexMap::with_hasher(Default::default());

        let mob_id = entity_ids.next();
        let mob_x = f32::TILE_SIZE * -0.5;
        let mob_y = f32::TILE_SIZE * 1.5;
        mobs.insert(mob_id, Mob::new(mob_x, mob_y));
        walkers.insert(mob_id, Walker { speed: 1.5 });
        impulses.insert(mob_id, Impulse { dx: 0.0, dy: 0.0 });

        let mob_id = entity_ids.next();
        let mob_x = f32::TILE_SIZE * -0.5;
        let mob_y = f32::TILE_SIZE * 2.5;
        mobs.insert(mob_id, Mob::new(mob_x, mob_y));
        walkers.insert(mob_id, Walker { speed: 1.5 });
        impulses.insert(mob_id, Impulse { dx: 0.0, dy: 0.0 });

        let mob_id = entity_ids.next();
        let mob_x = f32::TILE_SIZE * 1.5;
        let mob_y = f32::TILE_SIZE * 1.5;
        mobs.insert(mob_id, Mob::new(mob_x, mob_y));
        walkers.insert(mob_id, Walker { speed: 1.5 });
        impulses.insert(mob_id, Impulse { dx: 0.0, dy: 0.0 });

        let mob_id = entity_ids.next();
        let mob_x = f32::TILE_SIZE * 1.5;
        let mob_y = f32::TILE_SIZE * 2.5;
        mobs.insert(mob_id, Mob::new(mob_x, mob_y));
        walkers.insert(mob_id, Walker { speed: 1.5 });
        impulses.insert(mob_id, Impulse { dx: 0.0, dy: 0.0 });

        let mut towers = Default::default();
        let mut towers_by_pos = build_towers_by_pos(&towers);
        let mut swallow_targeters = Default::default();
        let mut missile_spawners = Default::default();
        // let mut missile_towers = IndexMap::with_hasher(Default::default());
        // let mut swallow_towers = Default::default();
        let mut swallows = Default::default();

        create_swallow_tower(
            &mut entity_ids,
            7,
            6,
            &mut towers,
            &mut towers_by_pos,
            &mut swallow_targeters,
            &mut swallows,
            &mut mobs,
            // We know there are no build orders to worry about at this stage
            &mut Default::default(),
        );

        create_swallow_tower(
            &mut entity_ids,
            14,
            14,
            &mut towers,
            &mut towers_by_pos,
            &mut swallow_targeters,
            &mut swallows,
            &mut mobs,
            // We know there are no build orders to worry about at this stage
            &mut Default::default(),
        );

        create_missile_tower(
            entity_ids.next(),
            3,
            6,
            &mut towers,
            &mut towers_by_pos,
            &mut missile_spawners,
            // We know there are no build orders to worry about at this stage
            &mut Default::default(),
        );

        create_missile_tower(
            entity_ids.next(),
            10,
            13,
            &mut towers,
            &mut towers_by_pos,
            &mut missile_spawners,
            // We know there are no build orders to worry about at this stage
            &mut Default::default(),
        );

        let mut falcons = Default::default();

        create_falcon_tower(
            &mut entity_ids,
            8,
            3,
            &mut towers,
            &mut towers_by_pos,
            &mut falcons,
            &mut mobs,
            // We know there are no build orders to worry about at this stage
            &mut Default::default(),
        );
        create_falcon_tower(
            &mut entity_ids,
            3,
            15,
            &mut towers,
            &mut towers_by_pos,
            &mut falcons,
            &mut mobs,
            // We know there are no build orders to worry about at this stage
            &mut Default::default(),
        );

        // for now, try making everything pusillanimous
        let mut pusillanimous = IndexMap::with_hasher(Default::default());
        for &entity in mobs.keys() {
            pusillanimous.insert(entity, Pusillanimous { duration: 0 });
        }

        // any initial towers start off operational
        for tower in towers.values_mut() {
            tower.status = TowerStatus::Operational;
        }

        World {
            game_speed: 1,
            core_state: CoreState {
                tick: 0,
                entity_ids,
                falcons,
                impulses,
                missile_spawners,
                mobs,
                pusillanimous,
                swallow_targeters,
                swallows,
                towers,
                towers_by_pos,
                walkers,
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
        for _ in 0..self.game_speed {
            self.core_state.tick += 1;
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
            self.progress_build();
            for (entity, mob) in &mut self.core_state.mobs {
                if self.core_state.walkers.contains_key(entity)
                    && mob.x >= (MAP_WIDTH * usize::TILE_SIZE + usize::TILE_SIZE / 2) as f32
                {
                    mob.x = -(f32::TILE_SIZE / 2.0);
                }
            }
        }
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
