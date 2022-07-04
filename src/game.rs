use std::cell::RefCell;
use std::rc::Rc;

use crate::dom::window;
use crate::particle_emitter::ParticleEmitter;
use crate::particle_shader::ParticleShader;
use crate::shapes::{Circle, Collides, Rectangle, Shape};
use crate::sprite::Sprite;
use crate::sprite_renderer::SpriteRenderer;
use crate::sprite_shader::SpriteShader;
use crate::terrain::{generate_terrain_contour, generate_terrain_mask, new_terrain_sprite};
use crate::texture::load_image_as_texture;
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
    terrain_dirty: bool,
    client_width: u32,
    client_height: u32,
    exiting: bool,
}

struct TankGameFlyweight {
    foreground_sprite: Sprite,
    foreground_mask_buffer: js_sys::Uint8Array,
    background_sprite: Sprite,
    game_state: GameState,
    sprite_renderer: SpriteRenderer,
    carriage_texture: Rc<WebGlTexture>,
    cannon_texture: Rc<WebGlTexture>,
    rocket_texture: Rc<WebGlTexture>,
    smoke_texture: Rc<WebGlTexture>,
    explosion_texture: Rc<WebGlTexture>,
    smoke_emitter: ParticleEmitter,
    explosion_emitter: ParticleEmitter,
    render_shapes: bool,
    sprite_shader: Rc<SpriteShader>,
    particle_shader: Rc<ParticleShader>,
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

    let beforeunload_game_clone = game.clone();
    let beforeunload_callback = Closure::wrap(Box::new(move || {
        let gl = get_rendering_context(&canvas).unwrap();
        let mut game = beforeunload_game_clone.borrow_mut();
        game.game_state.exiting = true;
        game.explosion_emitter.delete(&gl);
        game.smoke_emitter.delete(&gl);
        game.sprite_renderer.delete(&gl);
        gl.delete_program(Some(&game.particle_shader.program));
        gl.delete_program(Some(&game.sprite_shader.program));
        gl.delete_texture(Some(&game.explosion_texture));
        gl.delete_texture(Some(&game.smoke_texture));
        gl.delete_texture(Some(&game.rocket_texture));
        gl.delete_texture(Some(&game.cannon_texture));
        gl.delete_texture(Some(&game.carriage_texture));
        gl.delete_texture(Some(&game.foreground_sprite.texture()));
        gl.delete_texture(Some(&game.background_sprite.texture()));
    }) as Box<dyn FnMut()>);
    window().set_onbeforeunload(Some(beforeunload_callback.as_ref().unchecked_ref()));
    beforeunload_callback.forget();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let loop_clone = game.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |t: &JsValue| {
        let mut game = loop_clone.borrow_mut();

        if game.game_state.exiting {
            return;
        }

        // timestamp is in milliseconds
        let timestamp = t.as_f64().unwrap();
        let dt = (timestamp - game.game_state.timestamp) / 1000.0;

        update(&mut *game, dt as f32);
        prepare_dirty_resources(&gl, &mut *game);
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

    let client_width = canvas.client_width() as u32;
    let client_height = canvas.client_height() as u32;

    let background_texture = load_image_as_texture(&gl, "assets/background.jpg")?;
    let foreground_texture = load_image_as_texture(&gl, "assets/ground.jpg")?;
    let carriage_texture = load_image_as_texture(&gl, "assets/carriage.png")?;
    let cannon_texture = load_image_as_texture(&gl, "assets/cannon.png")?;
    let rocket_texture = load_image_as_texture(&gl, "assets/rocket.png")?;
    let smoke_texture = load_image_as_texture(&gl, "assets/smoke.png")?;
    let explosion_texture = load_image_as_texture(&gl, "assets/explosion.png")?;

    let sprite_shader = Rc::new(SpriteShader::new(gl)?);
    let sprite_renderer = SpriteRenderer::new(gl, sprite_shader.clone())?;

    let particle_shader = Rc::new(ParticleShader::new(gl)?);
    let mut smoke_emitter =
        ParticleEmitter::new(gl, smoke_texture.clone(), particle_shader.clone())?;
    smoke_emitter.initial_particle_life_seconds = 2.;
    smoke_emitter.initial_particle_scale = 25.;
    smoke_emitter.initial_particle_color = [0.2, 0.2, 0.2, 1.0];
    smoke_emitter.max_particle_offset = Vec3::new(10., 10., 0.);
    smoke_emitter.max_particles = 200;
    smoke_emitter.spawn_frequency_hz = 0.;

    let mut explosion_emitter =
        ParticleEmitter::new(gl, explosion_texture.clone(), particle_shader.clone())?;

    explosion_emitter.emitter_life_seconds = 2.;
    explosion_emitter.initial_particle_life_seconds = 1.;
    explosion_emitter.initial_particle_scale = 100.;
    explosion_emitter.initial_particle_color = [1., 1., 1., 1.];
    explosion_emitter.max_particle_offset = Vec3::new(50., 40., 0.);
    explosion_emitter.max_particles = 100;
    explosion_emitter.spawn_frequency_hz = 0.;

    let mut terrain_contour = js_sys::Float32Array::new_with_length(client_width as u32);
    generate_terrain_contour(&mut terrain_contour, client_height as f32);

    let player_positions = [
        (0.15f32 * client_width as f32) as u32,
        (0.3f32 * client_width as f32) as u32,
        (0.5f32 * client_width as f32) as u32,
        (0.75f32 * client_width as f32) as u32,
    ];

    let player_colors = [
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 0.0, 1.0, 1.0],
    ];

    // Flatten the terrain under the player positions
    for position in player_positions {
        let start = (position - 50).max(0);
        let end = (position + 50).min(client_width as u32);
        let height = terrain_contour.get_index(start);
        for i in start..end {
            terrain_contour.set_index(i, height);
        }
    }

    let buffer_size = (client_width * client_height * 4) as u32;
    let mut foreground_mask_buffer = js_sys::Uint8Array::new_with_length(buffer_size);

    let foreground_sprite = new_terrain_sprite(
        gl,
        foreground_texture,
        &mut foreground_mask_buffer,
        &terrain_contour,
        client_width,
        client_height,
    )?;

    let mut background_sprite = Sprite::new(gl, background_texture)?;
    background_sprite.global_scale = Vec3::new(client_width as f32, client_height as f32, 1.0);
    background_sprite.update();

    let projection = Mat4::orthographic(
        0.0,
        client_width as f32,
        client_height as f32,
        0.0,
        -1.0,
        1.0,
    );

    // preload shaders with projection as it won't change
    // This is an optimization to avoid a uniform call every frame
    gl.use_program(Some(&sprite_shader.program));
    gl.uniform_matrix4fv_with_f32_array(
        Some(&sprite_shader.projection_matrix_uniform),
        false,
        projection.data(),
    );
    gl.use_program(None);

    gl.use_program(Some(&particle_shader.program));

    gl.uniform_matrix4fv_with_f32_array(
        Some(&particle_shader.projection_matrix_uniform),
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
            cannon_sprite: Sprite::new_with_color(gl, cannon_texture.clone(), player_colors[0])?,
            carriage_sprite: Sprite::new_with_color(
                gl,
                carriage_texture.clone(),
                player_colors[0],
            )?,
            cannon_power: 200,
        },
        Player {
            id: 1,
            is_alive: true,
            terrain_position: player_positions[1],
            cannon_angle: 45.0,
            cannon_sprite: Sprite::new_with_color(gl, cannon_texture.clone(), player_colors[1])?,
            carriage_sprite: Sprite::new_with_color(
                gl,
                carriage_texture.clone(),
                player_colors[1],
            )?,
            cannon_power: 200,
        },
        Player {
            id: 2,
            is_alive: true,
            terrain_position: player_positions[2],
            cannon_angle: 45.0,
            cannon_sprite: Sprite::new_with_color(gl, cannon_texture.clone(), player_colors[2])?,
            carriage_sprite: Sprite::new_with_color(
                gl,
                carriage_texture.clone(),
                player_colors[2],
            )?,
            cannon_power: 200,
        },
        Player {
            id: 3,
            is_alive: true,
            terrain_position: player_positions[3],
            cannon_angle: 45.0,
            cannon_sprite: Sprite::new_with_color(gl, cannon_texture.clone(), player_colors[3])?,
            carriage_sprite: Sprite::new_with_color(
                gl,
                carriage_texture.clone(),
                player_colors[3],
            )?,
            cannon_power: 200,
        },
    ];

    for player in &mut players {
        reposition_player(player, &terrain_contour);
    }

    let game_state = GameState {
        timestamp: 0.0,
        current_player: 0,
        terrain_contour,
        rocket: None,
        players,
        terrain_dirty: false,
        client_width,
        client_height,
        exiting: false,
    };

    update_ui(&game_state);

    Ok(TankGameFlyweight {
        foreground_sprite,
        foreground_mask_buffer,
        background_sprite,
        game_state,
        sprite_renderer,
        rocket_texture,
        cannon_texture,
        carriage_texture,
        smoke_texture,
        explosion_texture,
        smoke_emitter,
        explosion_emitter,
        sprite_shader,
        particle_shader,
        render_shapes: false,
    })
}

fn reposition_player(player: &mut Player, terrain_contour: &Float32Array) {
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

fn create_rocket(
    texture: Rc<WebGlTexture>,
    mask: Rc<WebGlTexture>,
    color: [f32; 4],
    cannon_angle: f32,
    cannon_power: f32,
    cannon_x: f32,
    cannon_y: f32,
    player_id: usize,
) -> Result<Rocket, JsValue> {
    // Add an offset make it look like the rocket is leaving the cannon
    let position = Vec3::new(cannon_x as f32 - 10.0, cannon_y - 15.0, 0.0);

    let power = cannon_power;
    let (sin, cos) = ((cannon_angle - 90.0).to_radians()).sin_cos();
    let vx = power * cos;
    let vy = power * sin;
    let velocity = Vec3::new(vx, vy, 0.0);

    let scale_factor = 1.0 / 8.0;
    let mut sprite = Sprite::new_with_mask(texture, mask)?;
    sprite.color = color;
    sprite.global_scale = Vec3::new(86.0 * scale_factor, 287.0 * scale_factor, 0.0);
    sprite.local_position = Vec3::new(
        (-86.0 * scale_factor) / 2.0,
        (-287.0 * scale_factor) / 2.0,
        0.0,
    );
    sprite.global_position = position;

    Ok(Rocket {
        player_id,
        sprite,
        velocity,
    })
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

                game.game_state.rocket = Some(
                    create_rocket(
                        game.rocket_texture.clone(),
                        player.cannon_sprite.mask(),
                        player.cannon_sprite.color,
                        player.cannon_angle,
                        player.cannon_power as f32,
                        x as f32,
                        y,
                        player.id,
                    )
                    .expect("Could not create rocket"),
                );
                game.smoke_emitter.reset();
                game.smoke_emitter.spawn_frequency_hz = 120.;
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

        let position_y =
            player.carriage_sprite.global_position.y() + player.carriage_sprite.global_scale.y();
        let terrain_height = game_state
            .terrain_contour
            .get_index(player.terrain_position);

        if position_y >= game_state.client_height as f32 {
            player.is_alive = false;
        } else if terrain_height > position_y {
            reposition_player(player, &game_state.terrain_contour);
        }
    }
}

fn is_rocket_in_bounds(rocket: &Rocket, canvas_width: u32, canvas_height: u32) -> bool {
    let position = &rocket.sprite.global_position;

    position.x() > 0.0
        && position.y() > 0.0
        && position.x() < canvas_width as f32
        && position.y() < canvas_height as f32
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
    for _ in 0..state.players.len() {
        state.current_player = (state.current_player + 1) % state.players.len();

        if state.players[state.current_player].is_alive {
            break;
        }
    }

    update_ui(state);
}

fn add_crater_to_terrain(
    terrain_contour: &mut Float32Array,
    crater_center_x: f32,
    crater_radius: f32,
) {
    let crater_start = (crater_center_x - crater_radius).floor() as u32 + 1;
    let crater_end = (crater_center_x + crater_radius).ceil() as u32 - 1;
    let crater_center_y = terrain_contour.get_index(crater_center_x as u32);

    for x in crater_start..crater_end {
        let dx = (crater_center_x - x as f32).abs();
        let dy = (crater_radius * crater_radius - dx * dx).sqrt();

        let current_terrain_height = terrain_contour.get_index(x);
        let new_height = crater_center_y + dy;

        if current_terrain_height < new_height {
            terrain_contour.set_index(x, new_height);
        }
    }
}

fn update(game: &mut TankGameFlyweight, dt: f32) {
    if let Some(rocket) = &mut game.game_state.rocket {
        update_rocket(rocket, dt);
        game.smoke_emitter.location = rocket.sprite.global_position
            - Vec3::new(
                game.smoke_emitter.initial_particle_scale / 2.,
                game.smoke_emitter.initial_particle_scale / 2.,
                0.,
            );

        if !is_rocket_in_bounds(
            rocket,
            game.game_state.client_width,
            game.game_state.client_height,
        ) {
            game.game_state.rocket = None;
            game.smoke_emitter.spawn_frequency_hz = 0.;
            next_turn(&mut game.game_state);
        } else if let Some(player) = rocket_collided(rocket, &game.game_state.players) {
            add_crater_to_terrain(
                &mut game.game_state.terrain_contour,
                game.game_state.players[player]
                    .carriage_sprite
                    .global_position
                    .x(),
                40.0f32,
            );
            game.game_state.terrain_dirty = true;

            game.game_state.players[player].is_alive = false;
            game.smoke_emitter.spawn_frequency_hz = 0.;
            game.explosion_emitter.location = game.game_state.players[player]
                .carriage_sprite
                .global_position
                - Vec3::new(
                    game.explosion_emitter.initial_particle_scale / 2.,
                    game.explosion_emitter.initial_particle_scale / 2.,
                    0.,
                );
            game.explosion_emitter.reset();
            game.explosion_emitter.spawn_frequency_hz = 50.;
            game.game_state.rocket = None;
            next_turn(&mut game.game_state);
        } else if rocket_hit_terrain(rocket, &game.game_state.terrain_contour) {
            add_crater_to_terrain(
                &mut game.game_state.terrain_contour,
                rocket.sprite.global_position.x(),
                40.0f32,
            );
            game.game_state.terrain_dirty = true;
            game.smoke_emitter.spawn_frequency_hz = 0.;
            game.explosion_emitter.location = rocket.sprite.global_position
                - Vec3::new(
                    game.explosion_emitter.initial_particle_scale / 2.,
                    game.explosion_emitter.initial_particle_scale / 2.,
                    0.,
                );
            game.explosion_emitter.reset();
            game.explosion_emitter.spawn_frequency_hz = 50.;
            game.game_state.rocket = None;
            next_turn(&mut game.game_state)
        }
    }

    update_players(&mut game.game_state);
    game.smoke_emitter.update(dt);
    game.explosion_emitter.update(dt);

    if !game.game_state.players[game.game_state.current_player].is_alive {
        next_turn(&mut game.game_state);
    }
}

fn prepare_dirty_resources(gl: &WebGl2RenderingContext, game: &mut TankGameFlyweight) {
    if game.game_state.terrain_dirty {
        let old_mask = game.foreground_sprite.mask().clone();

        let new_mask = generate_terrain_mask(
            gl,
            &mut game.foreground_mask_buffer,
            &game.game_state.terrain_contour,
            game.game_state.client_width,
            game.game_state.client_height,
        )
        .expect("Could not create terrain mask");

        game.foreground_sprite.set_mask(new_mask.clone());
        gl.delete_texture(Some(&old_mask));
        game.game_state.terrain_dirty = false;
    }
}

fn render_shape(
    gl: &WebGl2RenderingContext,
    shape: &Shape,
    texture: Rc<WebGlTexture>,
    game: &TankGameFlyweight,
) {
    let mut sprite = Sprite::new(gl, texture).expect("Could not create shape sprite");
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

    gl.blend_func(
        WebGl2RenderingContext::SRC_ALPHA,
        WebGl2RenderingContext::ONE,
    );
    game.smoke_emitter.render(gl);
    game.explosion_emitter.render(gl);
}
