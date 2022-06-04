use std::rc::Rc;

use js_sys::Float32Array;
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{texture::create_rgba_texture_from_array_buffer_view, sprite::Sprite, vector::Vec3};

pub fn new_terrain_sprite(
    gl: &WebGl2RenderingContext,
    foreground_texture: Rc<WebGlTexture>,
    foreground_mask_buffer: &mut js_sys::Uint8Array,
    terrain_contour: &Float32Array,
    client_width: u32,
    client_height: u32)
-> Result<Sprite, JsValue> {

    let foreground_mask_texture = generate_terrain_mask(
        gl,
        foreground_mask_buffer,
        terrain_contour,
        client_width,
        client_height
    )?;

    let mut terrain_sprite = Sprite::new_with_mask(foreground_texture, foreground_mask_texture)?;
    terrain_sprite.global_scale = Vec3::new(
        client_width as f32,
        client_height as f32,
        1.0,
    );
    terrain_sprite.update();
    Ok(terrain_sprite)
}

pub fn generate_terrain_mask(
    gl: &WebGl2RenderingContext,
    foreground_mask_buffer: &mut js_sys::Uint8Array,
    terrain_contour: &Float32Array,
    client_width: u32,
    client_height: u32
) -> Result<Rc<WebGlTexture>, JsValue> {

    generate_foreground_mask_buffer(
        foreground_mask_buffer,
        terrain_contour,
        client_width,
        client_height
    );

    create_rgba_texture_from_array_buffer_view(
        gl,
        client_width,
        client_height,
        foreground_mask_buffer,
    )
}

fn contour_function(x: f32, max_y: f32, a: f32, b: f32, c: f32) -> f32 {
    let peak_height = 100.0;
    let flatness = 70.0;
    let offset = max_y / 1.33;

    peak_height / a * (x / flatness * a + a).sin()
        + peak_height / b * (x / flatness * b + b).sin()
        + peak_height / c * (x / flatness * c + c).sin()
        + offset
}

pub fn generate_terrain_contour(contour: &mut js_sys::Float32Array, max_height: f32) {
    let a = rand::random::<f32>() + 1.0;
    let b = rand::random::<f32>() + 2.0;
    let c = rand::random::<f32>() + 2.0;

    for i in 0..contour.length() {
        let height = contour_function(i as f32, max_height, a, b, c);
        contour.set_index(i, height)
    }
}

fn generate_foreground_mask_buffer(
    buffer: &mut js_sys::Uint8Array,
    contour: &js_sys::Float32Array,
    width: u32,
    height: u32,
) {
    for i in 0..width {
        let contour_height = contour.get_index(i);
        for j in 0..height {
            let index = 4 * (j * width + i) as u32;

            let color = match j >= contour_height as u32 {
                true => 255u8,
                false => 0u8,
            };

            buffer.set_index(index, color);
            buffer.set_index(index + 1, color);
            buffer.set_index(index + 2, color);
            buffer.set_index(index + 3, color);
        }
    }
}

