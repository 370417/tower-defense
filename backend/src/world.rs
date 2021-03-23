use fnv::FnvBuildHasher;
use indexmap::IndexMap;
use wasm_bindgen::prelude::*;

use crate::{
    explosion::{Explosion, Impulse},
    graphics::{create_mob, create_tower, render_mob_position},
    map::{
        distances::{
            calc_dist_from_exit, generate_dist_from_entrance, generate_dist_from_exit, Distances,
        },
        parse, render_map, true_row_col, Constants, Tile, MAP_0, MAP_WIDTH, TRUE_MAP_WIDTH,
    },
    missile::{Missile, MissileTower},
    mob::Mob,
    smoke::SmokeTrail,
    swallow::Swallow,
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
    pub impulses: Map<u32, Impulse>,
    #[wasm_bindgen(skip)]
    pub map: Vec<Tile>,
    #[wasm_bindgen(skip)]
    pub missile_towers: Map<u32, MissileTower>,
    #[wasm_bindgen(skip)]
    pub missiles: Map<u32, Missile>,
    #[wasm_bindgen(skip)]
    pub mobs: Map<u32, Mob>,
    #[wasm_bindgen(skip)]
    pub smoke_trails: Map<u32, SmokeTrail>,
    #[wasm_bindgen(skip)]
    pub swallows: Map<u32, Swallow>,
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

        create_mob(mob_id);
        render_mob_position(mob_id, mob_x, mob_y);

        let mob_id = entity_ids.next();
        let mob_x = f32::TILE_SIZE * -0.5;
        let mob_y = f32::TILE_SIZE * 2.5;
        mobs.insert(mob_id, Mob::new(mob_x, mob_y));
        walkers.insert(mob_id, Walker { speed: 1.5 });
        impulses.insert(mob_id, Impulse { dx: 0.0, dy: 0.0 });

        create_mob(mob_id);
        render_mob_position(mob_id, mob_x, mob_y);

        let mob_id = entity_ids.next();
        let mob_x = f32::TILE_SIZE * 1.5;
        let mob_y = f32::TILE_SIZE * 1.5;
        mobs.insert(mob_id, Mob::new(mob_x, mob_y));
        walkers.insert(mob_id, Walker { speed: 1.5 });
        impulses.insert(mob_id, Impulse { dx: 0.0, dy: 0.0 });

        create_mob(mob_id);
        render_mob_position(mob_id, mob_x, mob_y);

        let mob_id = entity_ids.next();
        let mob_x = f32::TILE_SIZE * 1.5;
        let mob_y = f32::TILE_SIZE * 2.5;
        mobs.insert(mob_id, Mob::new(mob_x, mob_y));
        walkers.insert(mob_id, Walker { speed: 1.5 });
        impulses.insert(mob_id, Impulse { dx: 0.0, dy: 0.0 });

        create_mob(mob_id);
        render_mob_position(mob_id, mob_x, mob_y);

        let mut missile_towers = IndexMap::with_hasher(Default::default());

        let tower_id = entity_ids.next();
        missile_towers.insert(
            tower_id,
            MissileTower {
                row: 3,
                col: 6,
                reload_countdown: 30,
                reload_cost: 60,
            },
        );
        create_tower(tower_id, 3, 6);

        let tower_id = entity_ids.next();
        missile_towers.insert(
            tower_id,
            MissileTower {
                row: 10,
                col: 13,
                reload_countdown: 30,
                reload_cost: 60,
            },
        );
        create_tower(tower_id, 10, 13);

        World {
            tick: 0,
            entity_ids,
            explosions: Default::default(),
            dist_from_entrance: generate_dist_from_entrance(&map),
            dist_from_exit: generate_dist_from_exit(&map),
            impulses,
            map,
            missile_towers,
            missiles: Default::default(),
            mobs,
            smoke_trails: Default::default(),
            swallows: Default::default(),
            walkers,
        }
    }

    pub fn update(&mut self) {
        self.tick += 1;
        self.remember_mob_positions();
        self.walk();
        self.fly_missiles();
        self.fly_swallows();
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
