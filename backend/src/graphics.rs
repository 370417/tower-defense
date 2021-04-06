use wasm_bindgen::prelude::*;

use crate::{smoke::SMOKE_TRAIL_LEN, world::World};

#[derive(Default)]
pub struct SpriteData {
    pub sprite_id: Vec<u8>,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub rotation: Vec<f32>,
    pub alpha: Vec<f32>,
    pub tint: Vec<u32>,
}

impl SpriteData {
    fn clear(&mut self) {
        self.sprite_id.clear();
        self.x.clear();
        self.y.clear();
        self.rotation.clear();
        self.alpha.clear();
        self.tint.clear();
    }

    pub fn push(&mut self, sprite_id: u8, x: f32, y: f32, rotation: f32, alpha: f32, tint: u32) {
        self.sprite_id.push(sprite_id);
        self.x.push(x);
        self.y.push(y);
        self.rotation.push(rotation);
        self.alpha.push(alpha);
        self.tint.push(tint);
    }
}

#[wasm_bindgen]
#[repr(u8)]
pub enum SpriteType {
    Swallow,
    Missile,
    Falcon,
    Walker,
    Indicator,
    MissileTower,
    TowerBase,
}

pub struct RopeData {}

#[wasm_bindgen]
impl World {
    pub fn sprite_count(&self) -> usize {
        self.sprite_data.sprite_id.len()
    }

    pub fn sprite_id(&self) -> *const u8 {
        self.sprite_data.sprite_id.as_ptr()
    }

    pub fn sprite_x(&self) -> *const f32 {
        self.sprite_data.x.as_ptr()
    }

    pub fn sprite_y(&self) -> *const f32 {
        self.sprite_data.y.as_ptr()
    }

    pub fn sprite_rotation(&self) -> *const f32 {
        self.sprite_data.rotation.as_ptr()
    }

    pub fn sprite_alpha(&self) -> *const f32 {
        self.sprite_data.alpha.as_ptr()
    }

    pub fn sprite_tint(&self) -> *const u32 {
        self.sprite_data.tint.as_ptr()
    }

    pub fn dump_sprite_data(&mut self, frame_fudge: f32) {
        // Order matters: sprites pushed first get rendered in the back.
        self.sprite_data.clear();

        for (id, targeter) in &self.swallow_targeters {
            targeter.dump(id, &mut self.sprite_data, &self.towers);
        }
        for (id, spawner) in &self.missile_spawners {
            spawner.dump(id, &mut self.sprite_data, &self.towers);
        }

        for swallow in self.swallow_after_images.values() {
            swallow.dump(&mut self.sprite_data, frame_fudge);
        }
        for (id, swallow) in &self.swallows {
            swallow.dump(id, &mut self.sprite_data, &self.mobs, frame_fudge);
        }
        for (id, missile) in &self.missiles {
            missile.dump(id, &mut self.sprite_data, &self.mobs, frame_fudge);
        }
        for (id, walker) in &self.walkers {
            walker.dump(id, &mut self.sprite_data, &self.mobs, frame_fudge);
        }
        for (id, falcon) in &self.falcons {
            falcon.dump(id, &mut self.sprite_data, &self.mobs, frame_fudge);
        }
        for (id, indicator) in &self.target_indicators {
            indicator.dump(id, &mut self.sprite_data, &self.mobs, frame_fudge);
        }

        // Shift everything half a pixel to account for the 1px borders between
        // tiles.
        for x in &mut self.sprite_data.x {
            *x += 0.5;
        }
        for y in &mut self.sprite_data.y {
            *y += 0.5;
        }
    }

    /// Call each renderable entity's render functions, which in turn call
    /// into the outside world (javascript) and render the game.
    ///
    /// This is mutable because of smoke trails: we pass data to js as an
    /// array, and we need to store it in the world so that it doesn't get
    /// dropped.
    pub fn render(&mut self, frame_fudge: f32) {
        for (&_id, smoke_trail) in &mut self.smoke_trails {
            smoke_trail.render(frame_fudge, self.tick);
        }
    }
}

#[wasm_bindgen]
extern "C" {
    pub fn render_path_tile(row: usize, col: usize);
    pub fn render_path_border(row: usize, col: usize, horizontal: bool);

    pub fn create_smoke_trail(id: u32, max_length: usize);
    pub fn render_smoke_trail(id: u32, x_ptr: *const f32, y_ptr: *const f32);
    pub fn recycle_smoke_trail(id: u32);

    pub fn create_explosion(id: u32, x: f32, y: f32);
    pub fn render_explosion(id: u32, radius: f32, alpha: f32);
    pub fn recycle_explosion(id: u32);

    pub fn render_range(x: f32, y: f32, radius: f32);
    pub fn recycle_range();
}
