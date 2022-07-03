use std::{rc::Rc, convert::TryInto};

use wasm_bindgen::JsValue;
use web_sys::{WebGlBuffer, WebGlVertexArrayObject, WebGl2RenderingContext};

use crate::{sprite_shader::SpriteShader, buffer::{create_square_buffer, create_texture_buffer}, particle_shader::ParticleShader};


/*
 * Define a Vertex Array Object that
 * can be used with shaders that take
 * two attributes:
 *  * a vec2 of vertex position
 *  * a vec2 of vertex position
 *  */
pub struct VAO {
    square_buffer: WebGlBuffer,
    texture_buffer: WebGlBuffer,
    pub vao: WebGlVertexArrayObject
}

impl VAO {
    pub fn new_with_sprite_shader(
        gl: &WebGl2RenderingContext,
        shader: Rc<SpriteShader>
    ) -> Result<VAO, JsValue> {
        let square_buffer = create_square_buffer(&gl)?;
        let texture_buffer = create_texture_buffer(&gl)?;

        let vao = gl
            .create_vertex_array()
            .ok_or_else(|| String::from("Could not create VAO"))?;

        gl.bind_vertex_array(Some(&vao));

        {
            let num_components = 2;
            let buffer_type = WebGl2RenderingContext::FLOAT;
            let normalized = false;
            let stride = 0;
            let offset = 0;
            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&square_buffer));
            gl.vertex_attrib_pointer_with_i32(
                shader.vertex_position_attrib.try_into().unwrap(),
                num_components,
                buffer_type,
                normalized,
                stride,
                offset,
            );
            gl.enable_vertex_attrib_array(shader.vertex_position_attrib.try_into().unwrap());
        }

        {
            let num_components = 2;
            let buffer_type = WebGl2RenderingContext::FLOAT;
            let normalized = false;
            let stride = 0;
            let offset = 0;
            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&texture_buffer));
            gl.vertex_attrib_pointer_with_i32(
                shader.vertex_texture_attrib.try_into().unwrap(),
                num_components,
                buffer_type,
                normalized,
                stride,
                offset,
            );
            gl.enable_vertex_attrib_array(shader.vertex_texture_attrib.try_into().unwrap());

            Ok(VAO {
                square_buffer,
                texture_buffer,
                vao
            })
        }
    }

    pub fn new_with_particle_shader(
        gl: &WebGl2RenderingContext,
        shader: Rc<ParticleShader>
    ) -> Result<VAO, JsValue> {
        let square_buffer = create_square_buffer(&gl)?;
        let texture_buffer = create_texture_buffer(&gl)?;

        let vao = gl
            .create_vertex_array()
            .ok_or_else(|| String::from("Could not create VAO"))?;

        gl.bind_vertex_array(Some(&vao));

        {
            let num_components = 2;
            let buffer_type = WebGl2RenderingContext::FLOAT;
            let normalized = false;
            let stride = 0;
            let offset = 0;
            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&square_buffer));
            gl.vertex_attrib_pointer_with_i32(
                shader.vertex_position_attrib.try_into().unwrap(),
                num_components,
                buffer_type,
                normalized,
                stride,
                offset,
            );
            gl.enable_vertex_attrib_array(shader.vertex_position_attrib.try_into().unwrap());
        }

        {
            let num_components = 2;
            let buffer_type = WebGl2RenderingContext::FLOAT;
            let normalized = false;
            let stride = 0;
            let offset = 0;
            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&texture_buffer));
            gl.vertex_attrib_pointer_with_i32(
                shader.vertex_texture_attrib.try_into().unwrap(),
                num_components,
                buffer_type,
                normalized,
                stride,
                offset,
            );
            gl.enable_vertex_attrib_array(shader.vertex_texture_attrib.try_into().unwrap());

            Ok(VAO {
                square_buffer,
                texture_buffer,
                vao
            })
        }
    }

    pub fn delete(&self, gl: &WebGl2RenderingContext) {
        gl.delete_buffer(Some(&self.square_buffer));
        gl.delete_buffer(Some(&self.texture_buffer));
        gl.delete_vertex_array(Some(&self.vao));
    }
}



