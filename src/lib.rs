
mod utils;
mod shader;
mod vector;
mod matrix;
mod buffer;

pub mod tank_game;

use wasm_bindgen::{prelude::*};
use web_sys::{console};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    // test
    console::log_1(&"Hello, World!".into());
}


