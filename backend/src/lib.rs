mod build;
mod collision;
mod distance;
mod ease;
mod explosion;
mod factory;
mod falcon;
mod graphics;
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
}
