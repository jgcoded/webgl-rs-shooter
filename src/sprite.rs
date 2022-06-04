use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::{WebGlTexture, WebGl2RenderingContext};

use crate::{matrix::Mat4, vector::Vec3, texture::create_rgba_texture_from_u8_array};

pub struct Sprite {
    texture: Rc<WebGlTexture>,
    mask: Rc<WebGlTexture>,
    pub color: [f32; 4],
    pub local_position: Vec3,
    pub global_scale: Vec3,
    pub global_position: Vec3,
    pub global_rotation: f32,
    model: Mat4
}

impl Sprite {
    pub fn new_with_mask(texture: Rc<WebGlTexture>, mask: Rc<WebGlTexture>) -> Result<Sprite, JsValue> {
        Ok(Sprite {
            texture,
            mask,
            color: [1.0, 1.0, 1.0, 1.0],
            local_position: Vec3::new(0.0, 0.0, 0.0),
            global_scale: Vec3::new(1.0, 1.0, 1.0),
            global_position: Vec3::new(0.0, 0.0, 0.0),
            global_rotation: 0.0,
            model: Mat4::identity()
        })
    }

    pub fn new_with_color(gl: &WebGl2RenderingContext, texture: Rc<WebGlTexture>, color: [f32; 4]) -> Result<Sprite, JsValue> {
        let mut sprite = Sprite::new(gl, texture)?;
        sprite.color = color;
        Ok(sprite)
    }

    pub fn new(gl: &WebGl2RenderingContext, texture: Rc<WebGlTexture>) -> Result<Sprite, JsValue> {
        let mask_array = [255, 255, 255, 255];
        let mask = create_rgba_texture_from_u8_array(&gl, 1, 1, &mask_array)?;
        Ok(Sprite {
            texture,
            mask,
            color: [1.0, 1.0, 1.0, 1.0],
            local_position: Vec3::new(0.0, 0.0, 0.0),
            global_scale: Vec3::new(1.0, 1.0, 1.0),
            global_position: Vec3::new(0.0, 0.0, 0.0),
            global_rotation: 0.0,
            model: Mat4::identity()
        })
    }

    pub fn update(&mut self) {
        self.model =
            // Scale to the image size
            Mat4::scale(self.global_scale.x(), self.global_scale.y(), self.global_scale.z())
            // Local transform to set local origin to rotation center
            * Mat4::translation(self.local_position.x(), self.local_position.y(), self.local_position.z())
            // Apply rotation
            * Mat4::rotate_z(self.global_rotation)
            // Translate to world position
            * Mat4::translation(self.global_position.x(), self.global_position.y(), self.global_position.z());
    }

    pub fn texture(&self) -> Rc<WebGlTexture> {
        self.texture.clone()
    }

    pub fn mask(&self) -> Rc<WebGlTexture> {
        self.mask.clone()
    }

    pub fn set_mask(&mut self, mask: Rc<WebGlTexture>) {
        self.mask = mask;
    }

    pub fn model(&self) -> &Mat4 {
        &self.model
    }
}
