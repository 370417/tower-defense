use wasm_bindgen::prelude::*;

use crate::world::World;

#[wasm_bindgen]
impl World {
    /// Call each renderable entity's render functions, which in turn call
    /// into the outside world (javascript) and render the game.
    pub fn render(&mut self, frame_fudge: f32) {
        for (&id, walker) in &self.walkers {
            walker.render(id, frame_fudge, &self);
        }
        for (&id, missile) in &self.missiles {
            missile.render(id, frame_fudge, &self);
        }
        for (&id, swallow) in &self.swallows {
            swallow.render(id, frame_fudge, &self);
        }
        for (&_id, smoke_trail) in &mut self.smoke_trails {
            smoke_trail.render(frame_fudge, self.tick);
        }
    }
}

#[wasm_bindgen]
extern "C" {
    pub fn render_path_tile(row: usize, col: usize);
    pub fn render_path_border(row: usize, col: usize, horizontal: bool);

    pub fn create_mob(id: u32);
    pub fn render_mob_position(id: u32, x: f32, y: f32);

    pub fn create_tower(id: u32, row: u32, col: u32);

    pub fn create_missile(id: u32);
    pub fn render_missile(id: u32, x: f32, y: f32, rotation: f32);
    pub fn recycle_missile(id: u32);

    pub fn create_smoke_trail(id: u32, max_length: usize);
    pub fn render_smoke_trail(id: u32, x_ptr: *const f32, y_ptr: *const f32);
    pub fn recycle_smoke_trail(id: u32);

    pub fn create_explosion(id: u32, x: f32, y: f32);
    pub fn render_explosion(id: u32, radius: f32, alpha: f32);
    pub fn recycle_explosion(id: u32);
}
