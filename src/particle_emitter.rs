use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::{WebGlTexture, WebGl2RenderingContext};

use crate::{particle_shader::ParticleShader, vector::Vec3, vao::VAO};

pub struct Particle {
    pub life: f32,
    pub scale: f32,
    pub offset: Vec3,
    pub color: [f32; 4]
}

pub struct ParticleEmitter {
    pub location: Vec3,
    pub texture: Rc<WebGlTexture>,
    pub spawn_frequency_hz: f32,
    pub initial_particle_life_seconds: f32,
    pub initial_particle_scale: f32,
    pub initial_particle_color: [f32; 4],
    pub max_particle_offset: Vec3,
    pub max_particles: usize,
    shader: Rc<ParticleShader>,
    vao: VAO,
    time: f32,
    last_spawn_time: f32,
    particles: Vec<Particle>,
}

impl ParticleEmitter {
    pub fn new(gl: &WebGl2RenderingContext, texture: Rc<WebGlTexture>, shader: Rc<ParticleShader>) -> Result<ParticleEmitter, JsValue> {
        Ok(ParticleEmitter {
            location: Vec3::new(0., 0., 0.),
            texture,
            spawn_frequency_hz: 1.,
            initial_particle_life_seconds: 0.,
            initial_particle_scale: 1.,
            initial_particle_color: [1.0, 1.0, 1.0, 1.0],
            max_particle_offset: Vec3::new(0., 0., 0.),
            max_particles: 100,
            shader: shader.clone(),
            vao: VAO::new_with_particle_shader(gl, shader)?,
            time: 0.,
            last_spawn_time: 0.,
            particles: Vec::new(),
        })
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;

        let initial_particle_life_seconds = self.initial_particle_life_seconds;
        self.particles.retain_mut(|p| {
            p.life -= dt;
            p.color[3] = p.life.max(0.) / initial_particle_life_seconds;
            p.life > 0.
        });

        let mut num_particles_to_spawn =
            ((self.time - self.last_spawn_time) * self.spawn_frequency_hz).floor() as usize;

        if num_particles_to_spawn > 0 {
            self.last_spawn_time = self.time;

            while num_particles_to_spawn > 0 && self.particles.len() < self.max_particles {
                let offset = Vec3::new(
                    self.max_particle_offset.x()*rand::random::<f32>() - self.max_particle_offset.x()/2.,
                    self.max_particle_offset.y()*rand::random::<f32>() - self.max_particle_offset.y()/2.,
                    0.
                );
                self.particles.push(Particle {
                    life: self.initial_particle_life_seconds,
                    scale: self.initial_particle_scale,
                    offset: self.location + offset,
                    color: self.initial_particle_color
                });
                num_particles_to_spawn -= 1;
            }
        }
    }

    pub fn render(&self, gl: &WebGl2RenderingContext) {
        gl.use_program(Some(&self.shader.program));

        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
        gl.uniform1i(Some(&self.shader.texture_sampler_uniform), 0);

        for particle in &self.particles {
            gl.uniform1f(Some(&self.shader.scale_uniform), particle.scale);
            gl.uniform3fv_with_f32_array(Some(&self.shader.offset_uniform), &particle.offset.data);
            gl.uniform4fv_with_f32_array(Some(&self.shader.color_uniform), &particle.color);

            gl.bind_vertex_array(Some(&self.vao.vao));
            {
                let offset = 0;
                let vertex_count = 6;
                gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, offset, vertex_count);
            }

            gl.bind_vertex_array(None);

        }
    }

    pub fn reset(&mut self) {
        self.time = 0.;
        self.last_spawn_time = 0.;
    }

    pub fn delete(&self, gl: &WebGl2RenderingContext) {
        self.vao.delete(gl);
    }
}
