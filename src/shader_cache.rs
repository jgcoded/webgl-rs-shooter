use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

use crate::sprite_shader::SpriteShader;

pub struct ShaderCache {
    sprite_shader: Rc<SpriteShader>,
}

impl ShaderCache {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<ShaderCache, JsValue> {
        Ok(ShaderCache {
            sprite_shader: Rc::new(SpriteShader::new(gl)?),
        })
    }

    pub fn sprite_shader(&self) -> Rc<SpriteShader> {
        self.sprite_shader.clone()
    }
}
