use std::rc::Rc;

use crate::sprite::Sprite;
use crate::sprite_shader::SpriteShader;
use crate::vao::VAO;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

pub struct SpriteRenderer {
    shader: Rc<SpriteShader>,
    vao: VAO
}

impl SpriteRenderer {
    pub fn new(
        gl: &WebGl2RenderingContext,
        shader: Rc<SpriteShader>,
    ) -> Result<SpriteRenderer, JsValue> {

        let vao = VAO::new_with_sprite_shader(gl, shader.clone())?;
        Ok(SpriteRenderer { shader, vao })
    }

    pub fn render(&self, gl: &WebGl2RenderingContext, sprite: &Sprite) {
        gl.use_program(Some(&self.shader.program));

        gl.uniform_matrix4fv_with_f32_array(
            Some(&self.shader.model_matrix_uniform),
            false,
            sprite.model().data(),
        );

        gl.bind_vertex_array(Some(&self.vao.vao));

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

    pub fn delete(&self, gl: &WebGl2RenderingContext) {
        self.vao.delete(gl);
    }
}
