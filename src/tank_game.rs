
use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

use crate::texture::{load_image_as_texture, create_rgba_texture_from_array_buffer_view, create_rgba_texture_from_u8_array};

use super::buffer::{create_square_buffer, create_texture_buffer};
use super::matrix::Mat4;
use super::shader::{compile_shader, link_program};
use super::utils::{set_panic_hook, get_rendering_context, get_canvas, request_animation_frame};
use js_sys::Math;
use wasm_bindgen::{prelude::*};
use web_sys::{WebGl2RenderingContext, console, WebGlProgram, WebGlUniformLocation, WebGlBuffer, HtmlCanvasElement, WebGlTexture, WebGlVertexArrayObject};

struct Player {
    terrain_position: u32
}

struct GameState {
    terrain_contour: js_sys::Float32Array,
    players: [Player; 4]
}

struct TankGameFlyweight {
    background_texture: Rc<WebGlTexture>,
    program: WebGlProgram,
    vertex_position_attrib: i32,
    vertex_texture_attrib: i32,
    projection_matrix_uniform: WebGlUniformLocation,
    model_view_matrix_uniform: WebGlUniformLocation,
    texture_sampler_uniform: WebGlUniformLocation,
    mask_sampler_uniform: WebGlUniformLocation,
    square_buffer: WebGlBuffer,
    texture_buffer: WebGlBuffer,
    projection: Mat4,
    model_view: Mat4,
    vertex_array_object: WebGlVertexArrayObject,
    foreground_mask_buffer: js_sys::Uint8Array,
    foreground_mask_texture: WebGlTexture,
    foreground_texture: Rc<WebGlTexture>,
    one_mask: WebGlTexture,
    game_state: GameState
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

    let background_texture = load_image_as_texture(&gl, "assets/tankgame/background.jpg")?;
    let foreground_texture = load_image_as_texture(&gl, "assets/tankgame/ground.jpg")?;

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
precision mediump float;

varying highp vec2 vTextureCoord;

uniform sampler2D uTextureSampler;
uniform sampler2D uMaskSampler;

void main(void) {
    vec4 color = texture2D(uTextureSampler, vTextureCoord);
    vec4 mask = texture2D(uMaskSampler, vTextureCoord);

    color = color * mask;

    if (color.a < 0.1)
        discard;

    gl_FragColor = color;
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
    "uTextureSampler")
        .ok_or_else(|| String::from("Could not get texture sampler uniform location"))?;

    let mask_sampler_uniform = gl.get_uniform_location(
        &program,
    "uMaskSampler"
    ).ok_or_else(|| String::from("Could not get mask sampler uniform location"))?;

    let square_buffer = create_square_buffer(&gl)?;
    let texture_buffer = create_texture_buffer(&gl)?;

    let vertex_array_object = gl.create_vertex_array()
        .ok_or_else(|| String::from("Could not create VAO"))?;

    gl.bind_vertex_array(Some(&vertex_array_object));

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
        let num_components = 2;
        let buffer_type = WebGl2RenderingContext::FLOAT;
        let normalized = false;
        let stride = 0;
        let offset = 0;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&texture_buffer));
        gl.vertex_attrib_pointer_with_i32(
            vertex_texture_attrib.try_into().unwrap(),
            num_components,
            buffer_type,
            normalized,
            stride,
            offset
        );
        gl.enable_vertex_attrib_array(vertex_texture_attrib.try_into().unwrap());
    }


    /*
    let field_of_view = 45.0;
    let aspect = (canvas.client_width() as f32) / (canvas.client_height() as f32);
    let near = 0.1;
    let far = 100.0;
    //let projection = Mat4::perspective(field_of_view, aspect, near, far);
    */
    //let projection = Mat4::orthographic(-1.0, 1.0, -1.0, 1.0, near, far);

    let projection = Mat4::orthographic(
        -canvas.client_width() as f32, 
        canvas.client_width() as f32, 
        -canvas.client_height() as f32,
        canvas.client_height() as f32,
        0.1,
        100.0);

    let model_view = Mat4::translation(0.0, 0.0, -6.0);

    let mut contour = js_sys::Float32Array::new_with_length(canvas.client_width() as u32);
    generate_terrain_contour(&mut contour, canvas.client_height() as f32);

    let game_state = GameState {
        terrain_contour: contour,
        players: [
        Player {
            terrain_position: (0.15f32 * canvas.client_width() as f32) as u32
        },
        Player {
            terrain_position: (0.3f32 * canvas.client_width() as f32) as u32
        },
        Player {
            terrain_position: (0.5f32 * canvas.client_width() as f32) as u32
        },
        Player {
            terrain_position: (0.75f32 * canvas.client_width() as f32) as u32
        }
        ]
    };

    for player in &game_state.players {
        let start = (player.terrain_position - 20).max(0);
        let end = (player.terrain_position + 20).min(canvas.client_width() as u32);
        let height = game_state.terrain_contour.get_index(start);
        for i in start..end {
            game_state.terrain_contour.set_index(i, height);
        }
    }

    let buffer_size = (canvas.client_width()*canvas.client_height()*4) as u32;
    let mut foreground_mask_buffer = js_sys::Uint8Array::new_with_length(buffer_size);

    generate_foreground_mask_buffer(
        &mut foreground_mask_buffer,
        &game_state.terrain_contour,
        canvas.client_width() as u32,
        canvas.client_height() as u32
    );

    let foreground_mask_texture = create_rgba_texture_from_array_buffer_view(
        gl,
        canvas.client_width(),
        canvas.client_height(),
        &mut foreground_mask_buffer
    )?;

    let mask_array = [255, 255, 255, 255];
    let one_mask = create_rgba_texture_from_u8_array(
        &gl, 1, 1, &mask_array
    )?;

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
        mask_sampler_uniform: mask_sampler_uniform,
        vertex_position_attrib: vertex_position_attrib,
        vertex_texture_attrib: vertex_texture_attrib,
        vertex_array_object: vertex_array_object,
        foreground_mask_texture: foreground_mask_texture,
        foreground_mask_buffer: foreground_mask_buffer,
        one_mask: one_mask,
        game_state: game_state,
        foreground_texture: foreground_texture
    })
}

fn contour_function(x: f32) -> f32 {
    0.1*(5.0*x).sin() + 0.8
}

fn generate_terrain_contour(
    contour: &mut js_sys::Float32Array,
    max_height: f32
) {
    for i in 0..contour.length() {
        let height = contour_function((i as f32) / (contour.length() as f32)) * max_height as f32;
        contour.set_index(i, height)
    }
}

fn generate_foreground_mask_buffer(
    buffer: &mut js_sys::Uint8Array,
    contour: &js_sys::Float32Array,
    width: u32,
    height: u32
) {
    for i in 0..width {
        let contour_height = contour.get_index(i);
        for j in 0..height {
            let index = 4*(j*width + i) as u32;
            
            let color = match j >= contour_height as u32 {
                true => 255u8,
                false => 0u8
            };

            buffer.set_index(index, color);
            buffer.set_index(index + 1, color);
            buffer.set_index(index + 2, color);
            buffer.set_index(index + 3, color);
        }
    }
}

fn update(game: &mut TankGameFlyweight, timestamp: f64) {
    let x = 0.0;// 0.8*(timestamp / 1000.0).cos();
    let y = 0.0; //0.8*(timestamp / 1000.0).sin();
    let translate = Mat4::translation(x as f32, y as f32, 0.0);

    let scale = Mat4::scale(640.0, 480.0, 1.0);// Mat4::scale(1024.0/768.0, 1.0, 1.0);

    let rotation = Mat4::rotate(0.0, 0.0, 0.0);

    game.model_view = rotation * scale * translate;
}

fn render(gl: &WebGl2RenderingContext, game: &TankGameFlyweight) {

    // TODO research perf of drawing square using triangle strip versus indexing via element buffer object

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear_depth(1.0);
    gl.enable(WebGl2RenderingContext::DEPTH_TEST);
    gl.depth_func(WebGl2RenderingContext::LEQUAL);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

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

    gl.bind_vertex_array(Some(&game.vertex_array_object));

    gl.active_texture(WebGl2RenderingContext::TEXTURE0);
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&game.background_texture));
    // Tell the shader we bound the texture to texture unit 0
    gl.uniform1i(Some(&game.texture_sampler_uniform), 0);

    gl.active_texture(WebGl2RenderingContext::TEXTURE1);
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&game.one_mask));
    gl.uniform1i(Some(&game.mask_sampler_uniform), 1);

    {
        let offset = 0;
        let vertex_count = 4;
        gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLE_STRIP,
            offset,
            vertex_count
        );
    }

    gl.active_texture(WebGl2RenderingContext::TEXTURE0);
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&game.foreground_texture));
    gl.uniform1i(Some(&game.texture_sampler_uniform), 0);

    gl.active_texture(WebGl2RenderingContext::TEXTURE1);
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&game.foreground_mask_texture));
    gl.uniform1i(Some(&game.mask_sampler_uniform), 1);

    {
        let offset = 0;
        let vertex_count = 4;
        gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLE_STRIP,
            offset,
            vertex_count
        );
    }

    gl.bind_vertex_array(None);
}

fn clean(gl: &WebGl2RenderingContext, game: &TankGameFlyweight) {
    gl.use_program(None);
    //gl.delete_texture(Some(&background_texture));
    gl.delete_buffer(Some(&game.square_buffer));
    gl.delete_buffer(Some(&game.texture_buffer));
    gl.delete_program(Some(&game.program));
}