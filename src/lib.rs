mod buffer;
mod dom;
mod matrix;
mod shader;
mod texture;
mod vector;
mod sprite;
mod shader_cache;
mod sprite_shader;
mod sprite_renderer;
mod shapes;

pub mod game;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
