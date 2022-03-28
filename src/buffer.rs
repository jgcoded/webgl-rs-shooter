use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

pub fn create_buffer(
    gl: &WebGl2RenderingContext,
    data: js_sys::Float32Array,
) -> Result<WebGlBuffer, JsValue> {
    let buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &data,
        WebGl2RenderingContext::STATIC_DRAW,
    );

    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
    Ok(buffer)
}

pub fn create_square_buffer(gl: &WebGl2RenderingContext) -> Result<WebGlBuffer, JsValue> {
    let positions = [0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0];

    let data = js_sys::Float32Array::new_with_length(positions.len() as u32);
    data.copy_from(&positions);

    create_buffer(gl, data)
}

pub fn create_texture_buffer(gl: &WebGl2RenderingContext) -> Result<WebGlBuffer, JsValue> {
    let texture_coordinates = [0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0];

    let data = js_sys::Float32Array::new_with_length(texture_coordinates.len() as u32);
    data.copy_from(&texture_coordinates);

    create_buffer(gl, data)
}
