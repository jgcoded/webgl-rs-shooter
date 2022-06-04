use std::convert::TryInto;
use std::rc::Rc;

use crate::buffer::{create_square_buffer, create_texture_buffer};
use crate::sprite::Sprite;
use crate::sprite_shader::SpriteShader;
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlVertexArrayObject};

pub struct SpriteRenderer {
    shader: Rc<SpriteShader>,
    vao: WebGlVertexArrayObject,
}

impl SpriteRenderer {
    pub fn new(
        gl: &WebGl2RenderingContext,
        shader: Rc<SpriteShader>,
    ) -> Result<SpriteRenderer, JsValue> {
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
        }

        Ok(SpriteRenderer { shader, vao })
    }

    pub fn render(&self, gl: &WebGl2RenderingContext, sprite: &Sprite) {
        gl.use_program(Some(&self.shader.program));

        gl.uniform_matrix4fv_with_f32_array(
            Some(&self.shader.model_matrix_uniform),
            false,
            sprite.model().data(),
        );

        gl.bind_vertex_array(Some(&self.vao));

        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&sprite.texture()));
        gl.uniform1i(Some(&self.shader.texture_sampler_uniform), 0);

        gl.active_texture(WebGl2RenderingContext::TEXTURE1);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&sprite.mask()));
        gl.uniform1i(Some(&self.shader.mask_sampler_uniform), 1);

        gl.uniform4fv_with_f32_array(Some(&self.shader.color_uniform), &sprite.color);

        {
            let offset = 0;
            let vertex_count = 6;
            gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, offset, vertex_count);
        }

        gl.bind_vertex_array(None);
    }
}
