mod build;
mod collision;
mod config;
mod distance;
mod ease;
mod explosion;
mod factory;
mod falcon;
mod graphics;
mod health;
mod map;
mod missile;
mod mob;
mod pusillanimous;
mod smoke;
mod spatial_index;
mod swallow;
mod targeting;
mod tower;
mod walker;
mod waves;
mod world;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    /// console.log can't handle too much data. dislay doesn't store old data,
    /// but should work better for displaying things every frame.
    fn display(s: &str);
}
