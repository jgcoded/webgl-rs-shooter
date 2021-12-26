
use std::convert::TryInto;

use super::buffer::{create_square_buffer, create_color_buffer};
use super::matrix::Mat4;
use super::shader::{compile_shader, link_program};
use super::utils::{set_panic_hook, get_rendering_context, get_canvas};
use wasm_bindgen::{prelude::*};
use web_sys::{WebGl2RenderingContext, console};


#[wasm_bindgen]
pub fn tank_game(canvas_id: &str) -> Result<(), JsValue> {
    set_panic_hook();
    console::log_1(&"Starting tank game".into());

    // https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial/Using_textures_in_WebGL
  
    let canvas = get_canvas(canvas_id)?;
    let gl = get_rendering_context(&canvas)?;

    let vertex_shader_source = r##"
attribute vec4 aVertexPosition;
attribute vec4 aVertexColor;
uniform mat4 uModelViewMatrix;
uniform mat4 uProjectionMatrix;
varying lowp vec4 vColor;
void main(void) {
  gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
  vColor = aVertexColor;
}
    "##;

    let fragment_shader_source = r##"
varying lowp vec4 vColor;
void main(void) {
    gl_FragColor = vColor;
}
    "##;

    let vertex_shader = compile_shader(&gl, 
        WebGl2RenderingContext::VERTEX_SHADER,
        vertex_shader_source)?;

    let fragment_shader = compile_shader(&gl,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        fragment_shader_source)?;

    let program = link_program(&gl, &vertex_shader, &fragment_shader)?;

    let vertex_position_attrib = gl.get_attrib_location(&program, "aVertexPosition");
    let vertex_color_attrib = gl.get_attrib_location(&program, "aVertexColor");

    let projection_matrix_uniform = gl.get_uniform_location(
            &program, 
        "uProjectionMatrix")
        .ok_or_else(|| String::from("Could not get project uniform location"))?;
    
    let model_view_matrix_uniform = gl.get_uniform_location(
        &program, 
        "uModelViewMatrix")
        .ok_or_else(|| String::from("Could not get model view uniform location"))?;

    let square_buffer = create_square_buffer(&gl)?;
    let color_buffer = create_color_buffer(&gl)?;

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear_depth(1.0);
    gl.enable(WebGl2RenderingContext::DEPTH_TEST);
    gl.depth_func(WebGl2RenderingContext::LEQUAL);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);


    let field_of_view = 45.0;
    let aspect = (canvas.client_width() as f32) / (canvas.client_height() as f32);
    let near = 0.1;
    let far = 100.0;
    let projection = Mat4::perspective(field_of_view, aspect, near, far);
    let model_view = Mat4::translation(0.0, 0.0, -6.0);

    {
        let num_components = 2;
        let buffer_type = WebGl2RenderingContext::FLOAT;
        let normalized = false;
        let stride = 0;
        let offset = 0;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&square_buffer));
        gl.vertex_attrib_pointer_with_i32(
            vertex_position_attrib.try_into().unwrap(), 
            num_components, 
            buffer_type,
            normalized,
            stride,
            offset
        );
        gl.enable_vertex_attrib_array(vertex_position_attrib.try_into().unwrap());
    }

    {

        let num_components = 4;
        let buffer_type = WebGl2RenderingContext::FLOAT;
        let normalized = false;
        let stride = 0;
        let offset = 0;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&color_buffer));
        gl.vertex_attrib_pointer_with_i32(
            vertex_color_attrib.try_into().unwrap(),
            num_components,
            buffer_type,
            normalized,
            stride,
            offset
        );
        gl.enable_vertex_attrib_array(vertex_color_attrib.try_into().unwrap());
    }

    gl.use_program(Some(&program));
    gl.uniform_matrix4fv_with_f32_array(
        Some(&projection_matrix_uniform),
        false,
        projection.data()
    );
    gl.uniform_matrix4fv_with_f32_array(
        Some(&model_view_matrix_uniform),
        false,
        model_view.data()
    );

    {
        let offset = 0;
        let vertex_count = 4;
        gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLE_STRIP,
            offset,
            vertex_count
        );
    }

    gl.use_program(None);
    gl.delete_buffer(Some(&square_buffer));
    gl.delete_buffer(Some(&color_buffer));
    gl.delete_program(Some(&program));

    Ok(())
}
