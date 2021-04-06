use fnv::FnvBuildHasher;
use indexmap::IndexMap;
use wasm_bindgen::prelude::*;

use crate::{
    explosion::{Explosion, Impulse},
    falcon::{create_falcon_tower, Falcon, TargetIndicator},
    graphics::{recycle_range, render_range, SpriteData},
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
    tower::{Range, Tower},
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

#[wasm_bindgen]
pub struct World {
    #[wasm_bindgen(skip)]
    pub tick: u32,
    #[wasm_bindgen(skip)]
    pub entity_ids: EntityIds,
    #[wasm_bindgen(skip)]
    pub explosions: Map<u32, Explosion>,
    #[wasm_bindgen(skip)]
    pub dist_from_entrance: Distances,
    #[wasm_bindgen(skip)]
    pub dist_from_exit: Distances,
    #[wasm_bindgen(skip)]
    pub falcons: Map<u32, Falcon>,
    #[wasm_bindgen(skip)]
    pub impulses: Map<u32, Impulse>,
    #[wasm_bindgen(skip)]
    pub map: Vec<Tile>,
    #[wasm_bindgen(skip)]
    pub missile_spawners: Map<u32, MissileSpawner>,
    #[wasm_bindgen(skip)]
    pub missiles: Map<u32, Missile>,
    #[wasm_bindgen(skip)]
    pub mobs: Map<u32, Mob>,
    #[wasm_bindgen(skip)]
    pub pusillanimous: Map<u32, Pusillanimous>,
    #[wasm_bindgen(skip)]
    pub smoke_trails: Map<u32, SmokeTrail>,
    #[wasm_bindgen(skip)]
    pub sprite_data: SpriteData,
    #[wasm_bindgen(skip)]
    pub swallow_after_images: Map<u32, SwallowAfterImage>,
    #[wasm_bindgen(skip)]
    pub swallow_targeters: Map<u32, SwallowTargeter>,
    #[wasm_bindgen(skip)]
    pub swallows: Map<u32, Swallow>,
    #[wasm_bindgen(skip)]
    pub target_indicators: Map<u32, TargetIndicator>,
    #[wasm_bindgen(skip)]
    pub threats: Map<u32, Threat>,
    #[wasm_bindgen(skip)]
    pub towers: Map<u32, Tower>,
    #[wasm_bindgen(skip)]
    pub walkers: Map<u32, Walker>,
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
            &mut swallow_targeters,
            &mut swallows,
            &mut mobs,
        );

        create_swallow_tower(
            &mut entity_ids,
            14,
            14,
            &mut towers,
            &mut swallow_targeters,
            &mut swallows,
            &mut mobs,
        );

        create_missile_tower(entity_ids.next(), 3, 6, &mut towers, &mut missile_spawners);

        create_missile_tower(
            entity_ids.next(),
            10,
            13,
            &mut towers,
            &mut missile_spawners,
        );

        let mut falcons = Default::default();

        create_falcon_tower(&mut entity_ids, 8, 3, &mut towers, &mut falcons, &mut mobs);
        create_falcon_tower(&mut entity_ids, 3, 15, &mut towers, &mut falcons, &mut mobs);

        // for now, try making everything pusillanimous
        let mut pusillanimous = IndexMap::with_hasher(Default::default());
        for &entity in mobs.keys() {
            pusillanimous.insert(entity, Pusillanimous { duration: 0 });
        }

        World {
            tick: 0,
            entity_ids,
            explosions: Default::default(),
            dist_from_entrance: generate_dist_from_entrance(&map),
            dist_from_exit: generate_dist_from_exit(&map),
            falcons,
            impulses,
            map,
            missile_spawners,
            missiles: Default::default(),
            mobs,
            pusillanimous,
            smoke_trails: Default::default(),
            sprite_data: Default::default(),
            swallow_after_images: Default::default(),
            swallow_targeters,
            swallows,
            target_indicators: Default::default(),
            threats: Default::default(),
            towers,
            walkers,
        }
    }

    pub fn update(&mut self) {
        self.tick += 1;
        self.remember_mob_positions();
        self.update_pusillanimity();
        self.walk();
        self.fly_missiles();
        self.fly_swallows();
        self.fade_swallow_after_images();
        self.fly_falcons();
        self.update_impulses();
        self.update_explosions();
        self.operate_missile_towers();
        self.update_smoke();
        for (entity, mob) in &mut self.mobs {
            if self.walkers.contains_key(entity)
                && mob.x >= (MAP_WIDTH * usize::TILE_SIZE + usize::TILE_SIZE / 2) as f32
            {
                mob.x = -(f32::TILE_SIZE / 2.0);
            }
        }
    }

    // 2 types of user inputs:
    // 1. stuff that affects the game and needs to be shared over the network
    // 2. stuff that just affects the ui, like this hover_map
    pub fn hover_map(&self, _player: u32, row: usize, col: usize) {
        for tower in self.towers.values() {
            if (row, col) == (tower.row, tower.col) {
                let (x, y) = tile_center(row, col);
                match tower.range {
                    Range::Circle { radius } => render_range(x, y, radius),
                }
                return;
            }
        }
        recycle_range();
    }
}

/// Stores the next available entity id (old ids are not reused)
pub struct EntityIds(u32);

impl EntityIds {
    pub fn next(&mut self) -> u32 {
        let id = self.0;
        self.0 += 1;
        id
    }
}
