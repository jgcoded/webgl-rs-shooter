use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlUniformLocation, WebGlProgram};

use crate::shader::{compile_shader, link_program};

pub struct SpriteShader {
    program: WebGlProgram,
    vertex_position_attrib: i32,
    vertex_texture_attrib: i32,
    projection_matrix_uniform: WebGlUniformLocation,
    model_matrix_uniform: WebGlUniformLocation,
    texture_sampler_uniform: WebGlUniformLocation,
    mask_sampler_uniform: WebGlUniformLocation,
}

impl SpriteShader {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<SpriteShader, JsValue> {
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

        let vertex_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            vertex_shader_source,
        )?;

        let fragment_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            fragment_shader_source,
        )?;

        let program = link_program(&gl, &vertex_shader, &fragment_shader)?;

        let vertex_position_attrib = gl.get_attrib_location(&program, "aVertexPosition");
        let vertex_texture_attrib = gl.get_attrib_location(&program, "aTextureCoord");

        let projection_matrix_uniform = gl
            .get_uniform_location(&program, "uProjectionMatrix")
            .ok_or_else(|| String::from("Could not get project uniform location"))?;

        let model_matrix_uniform = gl
            .get_uniform_location(&program, "uModelMatrix")
            .ok_or_else(|| String::from("Could not get model uniform location"))?;

        let texture_sampler_uniform = gl
            .get_uniform_location(&program, "uTextureSampler")
            .ok_or_else(|| String::from("Could not get texture sampler uniform location"))?;

        let mask_sampler_uniform = gl
            .get_uniform_location(&program, "uMaskSampler")
            .ok_or_else(|| String::from("Could not get mask sampler uniform location"))?;

        Ok(SpriteShader {
            program,
            vertex_position_attrib,
            vertex_texture_attrib,
            projection_matrix_uniform,
            model_matrix_uniform,
            texture_sampler_uniform,
            mask_sampler_uniform
        })
    }

    pub fn program(&self) -> &WebGlProgram {
        &self.program
    }

    pub fn vertex_position_attrib(&self) -> i32 {
        self.vertex_position_attrib
    }

    pub fn vertex_texture_attrib(&self) -> i32 {
        self.vertex_texture_attrib
    }

    pub fn projection_matrix_uniform(&self) -> &WebGlUniformLocation {
        &self.projection_matrix_uniform
    }

    pub fn model_matrix_uniform(&self) -> &WebGlUniformLocation {
        &self.model_matrix_uniform
    }

    pub fn texture_sampler_uniform(&self) -> &WebGlUniformLocation {
        &self.texture_sampler_uniform
    }

    pub fn texture_sampler_id(&self) -> i32 {
        0
    }

    pub fn mask_sampler_uniform(&self) -> &WebGlUniformLocation {
        &self.mask_sampler_uniform
    }

    pub fn mask_sampler_id(&self) -> i32 {
        1
    }

}
