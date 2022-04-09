use std::rc::Rc;

use web_sys::WebGlTexture;

use crate::{matrix::Mat4, vector::Vec3};

pub struct Sprite {
    texture: Rc<WebGlTexture>,
    mask: Rc<WebGlTexture>,
    pub local_position: Vec3,
    pub global_scale: Vec3,
    pub global_position: Vec3,
    pub global_rotation: f32,
    model: Mat4
}

impl Sprite {
    pub fn new(texture: Rc<WebGlTexture>, mask: Rc<WebGlTexture>) -> Sprite {
        Sprite {
            texture,
            mask,
            local_position: Vec3::new(0.0, 0.0, 0.0),
            global_scale: Vec3::new(1.0, 1.0, 1.0),
            global_position: Vec3::new(0.0, 0.0, 0.0),
            global_rotation: 0.0,
            model: Mat4::identity()
        }
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

    pub fn model(&self) -> &Mat4 {
        &self.model
    }
}
