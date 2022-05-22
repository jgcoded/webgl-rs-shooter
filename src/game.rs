use std::cell::RefCell;
use std::rc::Rc;

use crate::dom::window;
use crate::shader_cache::ShaderCache;
use crate::shapes::{Circle, Collides, Rectangle, Shape};
use crate::sprite::Sprite;
use crate::sprite_renderer::SpriteRenderer;
use crate::texture::{
    create_rgba_texture_from_array_buffer_view, create_rgba_texture_from_u8_array,
    load_image_as_texture,
};
use crate::ui::{post_ui_state, Ui};
use crate::vector::Vec3;

use super::dom::{get_canvas, get_rendering_context, request_animation_frame, set_panic_hook};
use super::matrix::Mat4;

use js_sys::Float32Array;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{console, HtmlCanvasElement, KeyboardEvent, WebGl2RenderingContext, WebGlTexture};

struct Player {
    id: usize,
    is_alive: bool,
    terrain_position: u32,
    carriage_sprite: Sprite,
    cannon_sprite: Sprite,
    cannon_angle: f32,
    cannon_power: u32,
}

struct Rocket {
    player_id: usize,
    sprite: Sprite,
    velocity: Vec3,
}

struct GameState {
    timestamp: f64,
    terrain_contour: js_sys::Float32Array,
    players: [Player; 4],
    current_player: usize,
    rocket: Option<Rocket>,
}

struct TankGameFlyweight {
    foreground_sprite: Sprite,
    background_sprite: Sprite,
    game_state: GameState,
    sprite_renderer: SpriteRenderer,
    rocket_texture: Rc<WebGlTexture>,
    render_shapes: bool,
}

#[wasm_bindgen]
pub fn start_game(canvas_id: &str) -> Result<(), JsValue> {
    let canvas = get_canvas(canvas_id)?;
    let gl = get_rendering_context(&canvas)?;

    let game = Rc::new(RefCell::new(initialize(&canvas, &gl)?));
    let keydown_game_clone = game.clone();
    let keydown_callback = Closure::wrap(Box::new(move |e: &KeyboardEvent| {
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

        let timestamp = t.as_f64().unwrap();
        let dt = (timestamp - game.game_state.timestamp) / 1000.0;

        update(&mut *game, dt as f32);
        render(&gl, &game);

        game.game_state.timestamp = timestamp;

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(&JsValue)>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn initialize(
    canvas: &HtmlCanvasElement,
    gl: &WebGl2RenderingContext,
) -> Result<TankGameFlyweight, JsValue> {
    set_panic_hook();
    console::log_1(&"Initializing tank game".into());

    let background_texture = load_image_as_texture(&gl, "assets/background.jpg")?;
    let foreground_texture = load_image_as_texture(&gl, "assets/ground.jpg")?;
    let carriage_texture = load_image_as_texture(&gl, "assets/carriage.png")?;
    let cannon_texture = load_image_as_texture(&gl, "assets/cannon.png")?;
    let rocket_texture = load_image_as_texture(&gl, "assets/rocket.png")?;
    let mask_array = [255, 255, 255, 255];
    let white_mask = create_rgba_texture_from_u8_array(&gl, 1, 1, &mask_array)?;
    let mask_array = [255, 0, 0, 255];
    let red_mask = create_rgba_texture_from_u8_array(&gl, 1, 1, &mask_array)?;
    let mask_array = [0, 255, 0, 255];
    let green_mask = create_rgba_texture_from_u8_array(&gl, 1, 1, &mask_array)?;
    let mask_array = [0, 0, 255, 255];
    let blue_mask = create_rgba_texture_from_u8_array(&gl, 1, 1, &mask_array)?;
    let mask_array = [255, 0, 255, 255];
    let purple_mask = create_rgba_texture_from_u8_array(&gl, 1, 1, &mask_array)?;

    let shader_cache = ShaderCache::new(gl)?;
    let sprite_renderer = SpriteRenderer::new(gl, shader_cache.sprite_shader())?;

    let mut terrain_contour = js_sys::Float32Array::new_with_length(canvas.client_width() as u32);
    generate_terrain_contour(&mut terrain_contour, canvas.client_height() as f32);

    let player_positions = [
        (0.15f32 * canvas.client_width() as f32) as u32,
        (0.3f32 * canvas.client_width() as f32) as u32,
        (0.5f32 * canvas.client_width() as f32) as u32,
        (0.75f32 * canvas.client_width() as f32) as u32,
    ];

    // Flatten the terrain under the player positions
    for position in player_positions {
        let start = (position - 50).max(0);
        let end = (position + 50).min(canvas.client_width() as u32);
        let height = terrain_contour.get_index(start);
        for i in start..end {
            terrain_contour.set_index(i, height);
        }
    }

    let buffer_size = (canvas.client_width() * canvas.client_height() * 4) as u32;
    let mut foreground_mask_buffer = js_sys::Uint8Array::new_with_length(buffer_size);

    generate_foreground_mask_buffer(
        &mut foreground_mask_buffer,
        &terrain_contour,
        canvas.client_width() as u32,
        canvas.client_height() as u32,
    );

    let foreground_mask_texture = create_rgba_texture_from_array_buffer_view(
        gl,
        canvas.client_width(),
        canvas.client_height(),
        &mut foreground_mask_buffer,
    )?;

    let mut foreground_sprite = Sprite::new(foreground_texture, foreground_mask_texture);
    foreground_sprite.global_scale = Vec3::new(
        canvas.client_width() as f32,
        canvas.client_height() as f32,
        1.0,
    );
    foreground_sprite.update();

    let mut background_sprite = Sprite::new(background_texture, white_mask);
    background_sprite.global_scale = Vec3::new(
        canvas.client_width() as f32,
        canvas.client_height() as f32,
        1.0,
    );
    background_sprite.update();

    let projection = Mat4::orthographic(
        0.0,
        canvas.client_width() as f32,
        canvas.client_height() as f32,
        0.0,
        -1.0,
        1.0,
    );

    let sprite_shader = shader_cache.sprite_shader();
    gl.use_program(Some(sprite_shader.program()));
    gl.uniform_matrix4fv_with_f32_array(
        Some(sprite_shader.projection_matrix_uniform()),
        false,
        projection.data(),
    );
    gl.use_program(None);

    let mut players = [
        Player {
            id: 0,
            is_alive: true,
            terrain_position: player_positions[0],
            cannon_angle: 45.0,
            cannon_sprite: Sprite::new(cannon_texture.clone(), red_mask.clone()),
            carriage_sprite: Sprite::new(carriage_texture.clone(), red_mask.clone()),
            cannon_power: 200,
        },
        Player {
            id: 1,
            is_alive: true,
            terrain_position: player_positions[1],
            cannon_angle: 45.0,
            cannon_sprite: Sprite::new(cannon_texture.clone(), green_mask.clone()),
            carriage_sprite: Sprite::new(carriage_texture.clone(), green_mask.clone()),
            cannon_power: 200,
        },
        Player {
            id: 2,
            is_alive: true,
            terrain_position: player_positions[2],
            cannon_angle: 45.0,
            cannon_sprite: Sprite::new(cannon_texture.clone(), blue_mask.clone()),
            carriage_sprite: Sprite::new(carriage_texture.clone(), blue_mask.clone()),
            cannon_power: 200,
        },
        Player {
            id: 3,
            is_alive: true,
            terrain_position: player_positions[3],
            cannon_angle: 45.0,
            cannon_sprite: Sprite::new(cannon_texture.clone(), purple_mask.clone()),
            carriage_sprite: Sprite::new(carriage_texture.clone(), purple_mask.clone()),
            cannon_power: 200,
        },
    ];

    for player in &mut players {
        let x = player.terrain_position;
        let y = terrain_contour.get_index(x) + -19.5;
        player.carriage_sprite.global_scale = Vec3::new(100.0, 39.0, 1.0);
        player.carriage_sprite.local_position = Vec3::new(-50.0, -19.5, 0.0);
        player.carriage_sprite.global_position = Vec3::new(x as f32, y, 0.0);
        player.carriage_sprite.update();
        player.cannon_sprite.global_scale = Vec3::new(20.0, 70.0, 1.0);
        player.cannon_sprite.local_position = Vec3::new(-10.0, -55.0, 0.0);
        player.cannon_sprite.global_position = Vec3::new(x as f32, y, 0.0);
        player.cannon_sprite.global_rotation = player.cannon_angle;
        player.cannon_sprite.update();
    }

    let game_state = GameState {
        timestamp: 0.0,
        current_player: 0,
        terrain_contour,
        rocket: None,
        players,
    };

    update_ui(&game_state);

    Ok(TankGameFlyweight {
        foreground_sprite,
        background_sprite,
        game_state,
        sprite_renderer,
        rocket_texture,
        render_shapes: false,
    })
}

fn create_rocket(
    texture: Rc<WebGlTexture>,
    mask: Rc<WebGlTexture>,
    cannon_angle: f32,
    cannon_power: f32,
    cannon_x: f32,
    cannon_y: f32,
    player_id: usize,
) -> Rocket {
    // Add an offset make it look like the rocket is leaving the cannon
    let position = Vec3::new(cannon_x as f32 - 10.0, cannon_y - 15.0, 0.0);

    let power = cannon_power;
    let (sin, cos) = ((cannon_angle - 90.0).to_radians()).sin_cos();
    let vx = power * cos;
    let vy = power * sin;
    let velocity = Vec3::new(vx, vy, 0.0);

    let scale_factor = 1.0 / 8.0;
    let mut sprite = Sprite::new(texture, mask);
    sprite.global_scale = Vec3::new(86.0 * scale_factor, 287.0 * scale_factor, 0.0);
    sprite.local_position = Vec3::new(
        (-86.0 * scale_factor) / 2.0,
        (-287.0 * scale_factor) / 2.0,
        0.0,
    );
    sprite.global_position = position;

    Rocket {
        player_id,
        sprite,
        velocity,
    }
}

fn contour_function(x: f32, max_y: f32, a: f32, b: f32, c: f32) -> f32 {
    let peak_height = 100.0;
    let flatness = 70.0;
    let offset = max_y / 2.0;

    peak_height / a * (x / flatness * a + a).sin()
        + peak_height / b * (x / flatness * b + b).sin()
        + peak_height / c * (x / flatness * c + c).sin()
        + offset
}

fn generate_terrain_contour(contour: &mut js_sys::Float32Array, max_height: f32) {
    let a = rand::random::<f32>() + 1.0;
    let b = rand::random::<f32>() + 2.0;
    let c = rand::random::<f32>() + 2.0;
    console::log_3(&JsValue::from(a), &JsValue::from(b), &JsValue::from(c));

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

fn handle_keyboard_input(game: &mut TankGameFlyweight, key_code: &str) -> bool {
    console::log_1(&key_code.into());
    let player = &mut game.game_state.players[game.game_state.current_player];

    match key_code {
        "ArrowLeft" => player.cannon_angle -= 2.0,
        "ArrowRight" => player.cannon_angle += 2.0,
        "ArrowUp" => {
            player.cannon_power += 5;
            update_ui(&game.game_state);
        }
        "ArrowDown" => {
            player.cannon_power -= 5;
            update_ui(&game.game_state);
        }
        " " => {
            if game.game_state.rocket.is_none() {
                let x = player.terrain_position;
                let y = game.game_state.terrain_contour.get_index(x);

                game.game_state.rocket = Some(create_rocket(
                    game.rocket_texture.clone(),
                    player.cannon_sprite.mask(),
                    player.cannon_angle,
                    player.cannon_power as f32,
                    x as f32,
                    y,
                    player.id,
                ));
            }
        }
        _ => return false,
    };

    // Keydown was handled
    true
}

fn update_players(game_state: &mut GameState) {
    for player in &mut game_state.players {
        if player.is_alive && game_state.current_player == player.id {
            player.cannon_sprite.global_rotation = player.cannon_angle;
            player.cannon_sprite.update()
        }
    }
}

fn is_rocket_in_bounds(rocket: &Rocket) -> bool {
    let position = &rocket.sprite.global_position;

    // TODO expose canvas width/height here
    position.x() > 0.0 && position.y() > 0.0 && position.x() < 1024.0 && position.y() < 768.0
}

fn update_rocket(rocket: &mut Rocket, dt: f32) {
    let gravity = Vec3::new(0.0, 150.0, 0.0);
    rocket.sprite.global_position += rocket.velocity.scaled(dt) + gravity.scaled(0.5 * dt * dt);
    rocket.velocity += gravity.scaled(dt);

    // Apply rotation
    let rocket_angle = 90.0
        - (-rocket.velocity.y())
            .atan2(rocket.velocity.x())
            .to_degrees();
    rocket.sprite.global_rotation = rocket_angle;
    rocket.sprite.update();
}

fn rocket_to_shape(rocket: &Rocket) -> Shape {
    // Use a square shape at the center
    let size = Vec3::new(
        rocket.sprite.global_scale.x() / 2.0,
        rocket.sprite.global_scale.x() / 2.0,
        0.0,
    );
    Shape::Rectangle(Rectangle {
        top_left: rocket.sprite.global_position - size,
        width: 2.0 * size.x(),
        height: 2.0 * size.y(),
    })
}

fn player_to_shape(player: &Player) -> Shape {
    let radius = player.carriage_sprite.global_scale.y() / 2.0;
    Shape::Circle(Circle {
        center: player.carriage_sprite.global_position,
        radius,
    })
}

fn rocket_collided(rocket: &Rocket, players: &[Player]) -> Option<usize> {
    let tip = rocket_to_shape(rocket);

    for player in players {
        if !player.is_alive || player.id == rocket.player_id {
            continue;
        }

        let player_shape = player_to_shape(player);
        if tip.intersects(&player_shape) {
            return Some(player.id);
        }
    }

    None
}

fn rocket_hit_terrain(rocket: &Rocket, terrain_contour: &Float32Array) -> bool {
    let x = rocket.sprite.global_position.x();
    let y = terrain_contour.get_index(x as u32);
    rocket.sprite.global_position.y() > y
}

fn update_ui(state: &GameState) {
    let current_player = &state.players[state.current_player];
    post_ui_state(&Ui {
        cannon_power: Some(current_player.cannon_power),
        current_player: Some(current_player.id),
        player_color: Some(String::from(match current_player.id {
            0 => "red",
            1 => "green",
            2 => "blue",
            3 => "purple",
            _ => "yelow",
        })),
    })
    .expect("Could not post UI state");
}

fn next_turn(state: &mut GameState) {
    loop {
        state.current_player = (state.current_player + 1) % state.players.len();

        if state.players[state.current_player].is_alive {
            break;
        }
    }

    update_ui(state);
}

fn update(game: &mut TankGameFlyweight, dt: f32) {
    update_players(&mut game.game_state);

    if let Some(rocket) = &mut game.game_state.rocket {
        update_rocket(rocket, dt);
        if !is_rocket_in_bounds(rocket) {
            game.game_state.rocket = None;
            next_turn(&mut game.game_state);
        } else if let Some(player) = rocket_collided(rocket, &game.game_state.players) {
            game.game_state.players[player].is_alive = false;
            game.game_state.rocket = None;
            next_turn(&mut game.game_state);
        } else if rocket_hit_terrain(rocket, &game.game_state.terrain_contour) {
            game.game_state.rocket = None;
            next_turn(&mut game.game_state)
        }
    }
}

fn render_shape(
    gl: &WebGl2RenderingContext,
    shape: &Shape,
    texture: Rc<WebGlTexture>,
    game: &TankGameFlyweight,
) {
    let mut sprite = Sprite::new(texture, game.background_sprite.mask());
    match shape {
        Shape::Rectangle(rectangle) => {
            sprite.global_position = rectangle.top_left;
            sprite.global_scale = Vec3::new(rectangle.width, rectangle.height, 1.0);
        }
        Shape::Circle(circle) => {
            sprite.global_position = circle.center;
            sprite.local_position = Vec3::new(-circle.radius, -circle.radius, 1.0);
            sprite.global_scale = Vec3::new(circle.radius * 2.0, circle.radius * 2.0, 1.0);
        }
    }

    sprite.update();
    game.sprite_renderer.render(gl, &sprite);
}

fn render(gl: &WebGl2RenderingContext, game: &TankGameFlyweight) {
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear_depth(1.0);
    gl.enable(WebGl2RenderingContext::BLEND);
    gl.blend_func(
        WebGl2RenderingContext::SRC_ALPHA,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );

    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    let renderer = &game.sprite_renderer;
    renderer.render(gl, &game.background_sprite);
    renderer.render(gl, &game.foreground_sprite);

    for player in &game.game_state.players {
        if !player.is_alive {
            continue;
        }

        renderer.render(gl, &player.cannon_sprite);
        renderer.render(gl, &player.carriage_sprite);

        // render shapes used in collision detection
        if game.render_shapes {
            let shape = player_to_shape(player);
            render_shape(gl, &shape, player.carriage_sprite.mask(), game);
        }
    }

    if let Some(rocket) = &game.game_state.rocket {
        renderer.render(gl, &rocket.sprite);
        if game.render_shapes {
            let shape = rocket_to_shape(rocket);
            render_shape(gl, &shape, rocket.sprite.mask(), game);
        }
    }
}
