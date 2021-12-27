
use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

use crate::texture::load_texture;

use super::buffer::{create_square_buffer, create_texture_buffer};
use super::matrix::Mat4;
use super::shader::{compile_shader, link_program};
use super::utils::{set_panic_hook, get_rendering_context, get_canvas, request_animation_frame};
use wasm_bindgen::{prelude::*};
use web_sys::{WebGl2RenderingContext, console, WebGlProgram, WebGlUniformLocation, WebGlBuffer, HtmlCanvasElement, WebGlTexture};

struct TankGameFlyweight {
    background_texture: Rc<WebGlTexture>,
    program: WebGlProgram,
    vertex_position_attrib: i32,
    vertex_texture_attrib: i32,
    projection_matrix_uniform: WebGlUniformLocation,
    model_view_matrix_uniform: WebGlUniformLocation,
    texture_sampler_uniform: WebGlUniformLocation,
    square_buffer: WebGlBuffer,
    texture_buffer: WebGlBuffer,
    projection: Mat4,
    model_view: Mat4,
}

#[wasm_bindgen]
pub fn tank_game(canvas_id: &str) -> Result<(), JsValue> {
    // https://rustwasm.github.io/wasm-bindgen/examples/request-animation-frame.html

    // https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial/Using_textures_in_WebGL
    let canvas = get_canvas(canvas_id)?;
    let gl = get_rendering_context(&canvas)?;


    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut game = initialize(&canvas, &gl)?;

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |t: &JsValue| {
        
        let timestamp = match t.as_f64() {
            Some(t) => t,
            _ => 0.0
        };
        
        update(&mut game, timestamp);

        render(&gl, &game);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(&JsValue)>));
    
    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn initialize(canvas: &HtmlCanvasElement, gl: &WebGl2RenderingContext) -> Result<TankGameFlyweight, JsValue> {
    set_panic_hook();
    console::log_1(&"Initializing tank game".into());

    let background_texture = load_texture(&gl, "assets/tankgame/background.jpg")?;

    let vertex_shader_source = r##"
attribute vec4 aVertexPosition;
attribute vec2 aTextureCoord;

uniform mat4 uModelViewMatrix;
uniform mat4 uProjectionMatrix;

varying highp vec2 vTextureCoord;

void main(void) {
  gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
  vTextureCoord = aTextureCoord;
}
    "##;

    let fragment_shader_source = r##"
varying highp vec2 vTextureCoord;

uniform sampler2D uSampler;

void main(void) {
    gl_FragColor = texture2D(uSampler, vTextureCoord);
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
    let vertex_texture_attrib = gl.get_attrib_location(&program, "aTextureCoord");

    let projection_matrix_uniform = gl.get_uniform_location(
            &program, 
        "uProjectionMatrix")
        .ok_or_else(|| String::from("Could not get project uniform location"))?;
    
    let model_view_matrix_uniform = gl.get_uniform_location(
        &program, 
        "uModelViewMatrix")
        .ok_or_else(|| String::from("Could not get model view uniform location"))?;

    let texture_sampler_uniform = gl.get_uniform_location(
        &program,
        "uSampler")
        .ok_or_else(|| String::from("Could not get texture sampler uniform location"))?;

    let square_buffer = create_square_buffer(&gl)?;
    let texture_buffer = create_texture_buffer(&gl)?;

    let field_of_view = 45.0;
    let aspect = (canvas.client_width() as f32) / (canvas.client_height() as f32);
    let near = 0.1;
    let far = 100.0;
    let projection = Mat4::perspective(field_of_view, aspect, near, far);
    let model_view = Mat4::translation(0.0, 0.0, -6.0);

    Ok(TankGameFlyweight {
        background_texture: background_texture,
        model_view: model_view,
        model_view_matrix_uniform: model_view_matrix_uniform,
        program: program,
        projection: projection,
        projection_matrix_uniform: projection_matrix_uniform,
        square_buffer: square_buffer,
        texture_buffer: texture_buffer,
        texture_sampler_uniform: texture_sampler_uniform,
        vertex_position_attrib: vertex_position_attrib,
        vertex_texture_attrib: vertex_texture_attrib
    })
}

fn update(game: &mut TankGameFlyweight, timestamp: f64) {
    let x = 0.0;// 0.8*(timestamp / 1000.0).cos();
    let y = 0.0; //0.8*(timestamp / 1000.0).sin();
    let translate = Mat4::translation(x as f32, y as f32, -2.4);

    let scale = Mat4::scale(1024.0/768.0, 1.0, 1.0);

    let rotation = Mat4::identity(); // Mat4::rotate(45.0*x as f32, 45.0*y as f32, 45.0);

    game.model_view = rotation * scale * translate;
}

fn render(gl: &WebGl2RenderingContext, game: &TankGameFlyweight) {

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear_depth(1.0);
    gl.enable(WebGl2RenderingContext::DEPTH_TEST);
    gl.depth_func(WebGl2RenderingContext::LEQUAL);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

    {
        let num_components = 2;
        let buffer_type = WebGl2RenderingContext::FLOAT;
        let normalized = false;
        let stride = 0;
        let offset = 0;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&game.square_buffer));
        gl.vertex_attrib_pointer_with_i32(
            game.vertex_position_attrib.try_into().unwrap(), 
            num_components, 
            buffer_type,
            normalized,
            stride,
            offset
        );
        gl.enable_vertex_attrib_array(game.vertex_position_attrib.try_into().unwrap());
    }

    {

        let num_components = 2;
        let buffer_type = WebGl2RenderingContext::FLOAT;
        let normalized = false;
        let stride = 0;
        let offset = 0;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&game.texture_buffer));
        gl.vertex_attrib_pointer_with_i32(
            game.vertex_texture_attrib.try_into().unwrap(),
            num_components,
            buffer_type,
            normalized,
            stride,
            offset
        );
        gl.enable_vertex_attrib_array(game.vertex_texture_attrib.try_into().unwrap());
    }

    gl.use_program(Some(&game.program));
    gl.uniform_matrix4fv_with_f32_array(
        Some(&game.projection_matrix_uniform),
        false,
        game.projection.data()
    );
    gl.uniform_matrix4fv_with_f32_array(
        Some(&game.model_view_matrix_uniform),
        false,
        game.model_view.data()
    );

    gl.active_texture(WebGl2RenderingContext::TEXTURE0);
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&game.background_texture));

    //   // Tell the shader we bound the texture to texture unit 0
    gl.uniform1i(Some(&game.texture_sampler_uniform), 0);

    {
        let offset = 0;
        let vertex_count = 4;
        gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLE_STRIP,
            offset,
            vertex_count
        );
    }
}

fn clean(gl: &WebGl2RenderingContext, game: &TankGameFlyweight) {
    gl.use_program(None);
    //gl.delete_texture(Some(&background_texture));
    gl.delete_buffer(Some(&game.square_buffer));
    gl.delete_buffer(Some(&game.texture_buffer));
    gl.delete_program(Some(&game.program));
}