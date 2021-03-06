mod buffer;
mod dom;
mod matrix;
mod shader;
mod texture;
mod vector;
mod vao;
mod sprite;
mod sprite_shader;
mod sprite_renderer;
mod particle_shader;
mod particle_emitter;
mod shapes;
mod terrain;
mod ui;

pub mod game;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
