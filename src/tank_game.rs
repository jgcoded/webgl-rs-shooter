
use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

use crate::texture::{load_image_as_texture, create_rgba_texture_from_array_buffer_view, create_rgba_texture_from_u8_array};
use crate::utils::window;
use crate::vector::Vec3;

use super::buffer::{create_square_buffer, create_texture_buffer};
use super::matrix::Mat4;
use super::shader::{compile_shader, link_program};
use super::utils::{set_panic_hook, get_rendering_context, get_canvas, request_animation_frame};
use js_sys::{Math, Array};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{WebGl2RenderingContext, console, WebGlProgram, WebGlUniformLocation, WebGlBuffer, HtmlCanvasElement, WebGlTexture, WebGlVertexArrayObject, KeyboardEvent};

struct Player {
    terrain_position: u32,
    model_matrix: Mat4,
    cannon_matrix: Mat4,
    color_mask: WebGlTexture,
    cannon_angle: f32
}

struct Rocket {
    local_model: Mat4,
    world_model: Mat4,
    position: Vec3,
    velocity: Vec3
}

struct GameState {
    terrain_contour: js_sys::Float32Array,
    players: [Player; 4],
    current_player: usize,
    rocket: Option<Rocket>
}

struct TankGameFlyweight {
    background_texture: Rc<WebGlTexture>,
    program: WebGlProgram,
    vertex_position_attrib: i32,
    vertex_texture_attrib: i32,
    projection_matrix_uniform: WebGlUniformLocation,
    model_matrix_uniform: WebGlUniformLocation,
    //view_matrix_uniform: WebGlUniformLocation,
    texture_sampler_uniform: WebGlUniformLocation,
    mask_sampler_uniform: WebGlUniformLocation,
    square_buffer: WebGlBuffer,
    texture_buffer: WebGlBuffer,
    projection: Mat4,
    background_model_matrix: Mat4,
    vertex_array_object: WebGlVertexArrayObject,
    foreground_mask_buffer: js_sys::Uint8Array,
    foreground_mask_texture: WebGlTexture,
    foreground_texture: Rc<WebGlTexture>,
    one_mask: WebGlTexture,
    carriage_texture: Rc<WebGlTexture>,
    cannon_texture: Rc<WebGlTexture>,
    game_state: GameState,
    //view_matrix: Mat4
    rocket_texture: Rc<WebGlTexture>
}

#[wasm_bindgen]
pub fn tank_game(canvas_id: &str) -> Result<(), JsValue> {
    // https://rustwasm.github.io/wasm-bindgen/examples/request-animation-frame.html

    // https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial/Using_textures_in_WebGL
    let canvas = get_canvas(canvas_id)?;
    let gl = get_rendering_context(&canvas)?;


    let game = Rc::new(RefCell::new(initialize(&canvas, &gl)?));
    let keydown_game_clone = game.clone();
    let keydown_callback = Closure::wrap(Box::new(move |e : &KeyboardEvent| {
        let mut game = keydown_game_clone.borrow_mut();
        let handled = handle_keyboard_input(&mut *game, e.key().as_str());

        if handled {
            e.prevent_default();
        }
    }) as Box<dyn FnMut(&KeyboardEvent)>);

    window().set_onkeydown(Some(keydown_callback.as_ref().unchecked_ref()));
    // Give ownership to the browser
    keydown_callback.forget();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let loop_clone = game.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |t: &JsValue| {
        let mut game = loop_clone.borrow_mut();
        let timestamp = match t.as_f64() {
            Some(t) => t,
            _ => 0.0
        };

        update(&mut *game, timestamp);

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
    let carriage_texture = load_image_as_texture(&gl, "assets/tankgame/carriage.png")?;
    let cannon_texture = load_image_as_texture(&gl, "assets/tankgame/cannon.png")?;
    let rocket_texture = load_image_as_texture(&gl, "assets/tankgame/rocket.png")?;

    let vertex_shader_source = r##"
attribute vec4 aVertexPosition;
attribute vec2 aTextureCoord;

uniform mat4 uModelMatrix;
uniform mat4 uProjectionMatrix;

varying highp vec2 vTextureCoord;

void main(void) {
  gl_Position = uProjectionMatrix * uModelMatrix * aVertexPosition;
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

    gl_FragColor = mask * color;
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
    
    let model_matrix_uniform = gl.get_uniform_location(
        &program, 
        "uModelMatrix")
        .ok_or_else(|| String::from("Could not get model uniform location"))?;

        /*
    let view_matrix_uniform = gl.get_uniform_location(
        &program, 
        "uViewMatrix")
        .ok_or_else(|| String::from("Could not get view uniform location"))?;
*/
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
    /*
    let projection = Mat4::orthographic(
        -canvas.client_width() as f32, 
        canvas.client_width() as f32, 
        -canvas.client_height() as f32,
        canvas.client_height() as f32,
        0.1,
        100.0);
    */
    let projection = Mat4::orthographic(
        0.0, 
        canvas.client_width() as f32, 
        canvas.client_height() as f32,
        0.0,
        -1.0,
        1.0);
    // https://learnopengl.com/In-Practice/2D-Game/Rendering-Sprites
    /*
    let view_matrix = Mat4::look_at(
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 0.0, 0.0), 
            Vec3::new(0.0, 1.0, 0.0)
    );
    */

    let translate = Mat4::translation(0.0, 0.0, 0.0);
    let scale = Mat4::scale(canvas.client_width() as f32, canvas.client_height() as f32, 0.0);// Mat4::scale(1024.0/768.0, 1.0, 1.0);
    //let rotation = Mat4::rotate(0.0, 0.0, 0.0);
    let background_model_matrix = scale*translate;
    
    let mut contour = js_sys::Float32Array::new_with_length(canvas.client_width() as u32);
    generate_terrain_contour(&mut contour, canvas.client_height() as f32);

    let mask_array = [255, 0, 0, 255];
    let red_mask = create_rgba_texture_from_u8_array(
        &gl, 1, 1, &mask_array
    )?;

    let mask_array = [0, 255, 0, 255];
    let green_mask = create_rgba_texture_from_u8_array(
        &gl, 1, 1, &mask_array
    )?;

    let mask_array = [0, 0, 255, 255];
    let blue_mask = create_rgba_texture_from_u8_array(
        &gl, 1, 1, &mask_array
    )?;

    let mask_array = [255, 0, 255, 255];
    let purple_mask = create_rgba_texture_from_u8_array(
        &gl, 1, 1, &mask_array
    )?;

    let mut game_state = GameState {
        current_player: 0,
        terrain_contour: contour,
        rocket: None,
        players: [
        Player {
            terrain_position: (0.15f32 * canvas.client_width() as f32) as u32,
            model_matrix: Mat4::identity(),
            color_mask: red_mask,
            cannon_matrix: Mat4::identity(),
            cannon_angle: 45.0
        },
        Player {
            terrain_position: (0.3f32 * canvas.client_width() as f32) as u32,
            model_matrix: Mat4::identity(),
            color_mask: green_mask,
            cannon_matrix: Mat4::identity(),
            cannon_angle: 45.0
        },
        Player {
            terrain_position: (0.5f32 * canvas.client_width() as f32) as u32,
            model_matrix: Mat4::identity(),
            color_mask: blue_mask,
            cannon_matrix: Mat4::identity(),
            cannon_angle: 45.0
        },
        Player {
            terrain_position: (0.75f32 * canvas.client_width() as f32) as u32,
            model_matrix: Mat4::identity(),
            color_mask: purple_mask,
            cannon_matrix: Mat4::identity(),
            cannon_angle: 45.0
        }
        ]
    };

    for player in &game_state.players {
        let start = (player.terrain_position - 50).max(0);
        let end = (player.terrain_position + 50).min(canvas.client_width() as u32);
        let height = game_state.terrain_contour.get_index(start);
        for i in start..end {
            game_state.terrain_contour.set_index(i, height);
        }
    }

    update_players(&mut game_state);


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
        background_model_matrix: background_model_matrix,
        model_matrix_uniform: model_matrix_uniform,
        //view_matrix_uniform: view_matrix_uniform,
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
        foreground_texture: foreground_texture,
        carriage_texture: carriage_texture,
        //view_matrix: view_matrix
        cannon_texture: cannon_texture,
        rocket_texture: rocket_texture
    })
}

fn create_rocket(position: Vec3, velocity: Vec3) -> Rocket {
    let rocket_model = Mat4::scale(86.0/8.0, 287.0/8.0, 0.0);
    // Move origin of rocket to center of image
    let rocket_model = rocket_model * Mat4::translation(-86.0/16.0, -287.0/16.0, 0.0);
    Rocket {
        local_model: rocket_model,
        position: position,
        velocity: velocity,
        world_model: Mat4::identity()
    }
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

fn handle_keyboard_input(game: &mut TankGameFlyweight, key_code: &str) -> bool {
    console::log_1(&key_code.into());
    let player = &mut game.game_state.players[game.game_state.current_player];

    match key_code {
        "ArrowLeft" => player.cannon_angle -= 2.0,
        "ArrowRight" => player.cannon_angle += 2.0,
        " " => {
            if game.game_state.rocket.is_none() {
                let x = player.terrain_position;
                let y = game.game_state.terrain_contour.get_index(x);

                // Add an offset make it look like the rocket is leaving the cannon
                let position = Vec3::new(x as f32 - 10.0, y - 15.0, 0.0);

                let rad_per_degree = std::f32::consts::PI / 180.0f32;
                let (sin, cos) = ((player.cannon_angle-90.0)*rad_per_degree).sin_cos();
                let vx = 10.0 * cos;
                let vy = 10.0 * sin;
                let velocity = Vec3::new(vx, vy, 0.0);
                game.game_state.rocket = Some(create_rocket(position, velocity));
            }
        },
        _ => return false
    };

    // Keydown was handled
    true
}

fn update_players(game_state: &mut GameState) {
    for player in &mut game_state.players {
        let x = player.terrain_position;
        let y = game_state.terrain_contour.get_index(x);

        // Scale to image size
        let model = Mat4::scale(100.0, 39.0, 1.0);

        // Move origin to image center
        let model = model * Mat4::translation(-50.0, -19.5, 0.0);

        // Local transform to have the bottom of the carriage touch the ground
        let model = model * Mat4::translation(0.0, -19.5, 0.0);

        // Translate to world position
        let model = model * Mat4::translation(x as f32, y, 0.0);

        player.model_matrix = model;

        // Scale to the image size
        let model = Mat4::scale(20.0, 70.0, 1.0);

        // Move origin to bottom center of cannon
        let model = model * Mat4::translation(-10.0, -55.0, 0.0);

        // Apply rotation
        let model = model * Mat4::rotate(0.0, 0.0, player.cannon_angle);

        // Move origin back
        let model = model * Mat4::translation(10.0, 55.0, 0.0);

        // local transform to move cannon to the carriage wheel center
        let model = model * Mat4::translation(-10.0, -75.0, 0.0);

        // Translate to world position
        let model = model * Mat4::translation(x as f32, y, 0.0);

        player.cannon_matrix = model;
    }
}

fn is_rocket_in_bounds(rocket: &Rocket) -> bool{
    let position = &rocket.position;

    // TODO expose canvas width/height here
    position.x() > 0.0 && position.y() > 0.0 &&
    position.x() < 1024.0 && position.y() < 768.0
}

fn update_rocket(rocket: &mut Rocket, timestamp: f64) {
    let model = rocket.local_model;

    let gravity = Vec3::new(0.0, 0.1, 0.0);
    rocket.velocity += gravity;
    rocket.position += rocket.velocity;

    // Rotate
    let degree_per_rad = 180.0f32 / std::f32::consts::PI;

    // Apply rotation
    let rocket_angle = 90.0- degree_per_rad * (-rocket.velocity.y()).atan2(rocket.velocity.x());
    let model = model * Mat4::rotate(0.0, 0.0, rocket_angle);

    // Translate
    let model = model * Mat4::translation(rocket.position.x(), rocket.position.y(), rocket.position.z());

    rocket.world_model = model;
}

fn update(game: &mut TankGameFlyweight, timestamp: f64) {
    update_players(&mut game.game_state);

    if let Some(rocket) = &mut game.game_state.rocket {
        if is_rocket_in_bounds(rocket) {
            update_rocket(rocket, timestamp);
        } else {
            game.game_state.rocket = None;
            game.game_state.current_player = (game.game_state.current_player + 1) % game.game_state.players.len();
        }
    }
}

fn render(gl: &WebGl2RenderingContext, game: &TankGameFlyweight) {

    // TODO research perf of drawing square using triangle strip versus indexing via element buffer object

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear_depth(1.0);
    gl.enable(WebGl2RenderingContext::BLEND);
    gl.blend_func(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA);
    //gl.enable(WebGl2RenderingContext::DEPTH_TEST);
    //gl.depth_func(WebGl2RenderingContext::LEQUAL);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT /* | WebGl2RenderingContext::DEPTH_BUFFER_BIT */);

    gl.use_program(Some(&game.program));

    gl.uniform_matrix4fv_with_f32_array(
        Some(&game.projection_matrix_uniform),
        false,
        game.projection.data()
    );
    gl.uniform_matrix4fv_with_f32_array(
        Some(&game.model_matrix_uniform),
        false,
        game.background_model_matrix.data()
    );

    gl.bind_vertex_array(Some(&game.vertex_array_object));

    render_texture_with_mask(gl, &game.background_texture, &game.texture_sampler_uniform, &game.one_mask, &game.mask_sampler_uniform);

    render_texture_with_mask(gl, &game.foreground_texture, &game.texture_sampler_uniform, &game.foreground_mask_texture, &game.mask_sampler_uniform);

    for player in &game.game_state.players {

        gl.uniform_matrix4fv_with_f32_array(
            Some(&game.model_matrix_uniform),
            false,
            player.cannon_matrix.data()
        );
        render_texture_with_mask(gl, &game.cannon_texture, &game.texture_sampler_uniform, &player.color_mask, &game.mask_sampler_uniform);

        gl.uniform_matrix4fv_with_f32_array(
            Some(&game.model_matrix_uniform),
            false,
            player.model_matrix.data()
        );
        render_texture_with_mask(gl, &game.carriage_texture, &game.texture_sampler_uniform, &player.color_mask, &game.mask_sampler_uniform);
    }

    if let Some(rocket) = &game.game_state.rocket {
        gl.uniform_matrix4fv_with_f32_array(
            Some(&game.model_matrix_uniform),
            false,
            rocket.world_model.data()
        );
        let player = &game.game_state.players[game.game_state.current_player];
        render_texture_with_mask(gl, &game.rocket_texture, &game.texture_sampler_uniform, &player.color_mask, &game.mask_sampler_uniform);
    }

    gl.bind_vertex_array(None);

}

// Assumes a shader program with texture sampler 0 and mask sampler 1
fn render_texture_with_mask(
    gl: &WebGl2RenderingContext,
    texture: &WebGlTexture,
    texture_sampler_uniform: &WebGlUniformLocation,
    mask: &WebGlTexture,
    mask_sampler_uniform: &WebGlUniformLocation
) {
    gl.active_texture(WebGl2RenderingContext::TEXTURE0);
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
    gl.uniform1i(Some(texture_sampler_uniform), 0);

    gl.active_texture(WebGl2RenderingContext::TEXTURE1);
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(mask));
    gl.uniform1i(Some(mask_sampler_uniform), 1);

    {
        let offset = 0;
        let vertex_count = 6;
        gl.draw_arrays(
            WebGl2RenderingContext::TRIANGLES,
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