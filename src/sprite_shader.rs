use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlUniformLocation, WebGlProgram};

use crate::shader::new_shader_program;

pub struct SpriteShader {
    pub program: WebGlProgram,
    pub vertex_position_attrib: i32,
    pub vertex_texture_attrib: i32,
    pub model_matrix_uniform: WebGlUniformLocation,
    pub projection_matrix_uniform: WebGlUniformLocation,
    pub texture_sampler_uniform: WebGlUniformLocation,
    pub mask_sampler_uniform: WebGlUniformLocation,
    pub color_uniform: WebGlUniformLocation
}

impl SpriteShader {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<SpriteShader, JsValue> {
        let vertex_shader_src = r##"
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

        let fragment_shader_src = r##"
        precision mediump float;

        varying highp vec2 vTextureCoord;

        uniform sampler2D uTextureSampler;
        uniform sampler2D uMaskSampler;
        uniform vec4 uColor;

        void main(void) {
            vec4 texture = texture2D(uTextureSampler, vTextureCoord);
            vec4 mask = texture2D(uMaskSampler, vTextureCoord);
            gl_FragColor = texture * mask * uColor;
        }
            "##;

        let program = new_shader_program(gl, vertex_shader_src, fragment_shader_src)?;

        let vertex_position_attrib = gl.get_attrib_location(&program, "aVertexPosition");
        let vertex_texture_attrib = gl.get_attrib_location(&program, "aTextureCoord");

        let model_matrix_uniform = gl
            .get_uniform_location(&program, "uModelMatrix")
            .ok_or_else(|| String::from("Could not get model uniform location"))?;

        let projection_matrix_uniform = gl
            .get_uniform_location(&program, "uProjectionMatrix")
            .ok_or_else(|| String::from("Could not get model uniform location"))?;
        let texture_sampler_uniform = gl
            .get_uniform_location(&program, "uTextureSampler")
            .ok_or_else(|| String::from("Could not get texture sampler uniform location"))?;

        let mask_sampler_uniform = gl
            .get_uniform_location(&program, "uMaskSampler")
            .ok_or_else(|| String::from("Could not get mask sampler uniform location"))?;

        let color_uniform = gl
            .get_uniform_location(&program, "uColor")
            .ok_or_else(|| String::from("Could not get mask sampler uniform location"))?;

        Ok(SpriteShader {
            program,
            vertex_position_attrib,
            vertex_texture_attrib,
            model_matrix_uniform,
            projection_matrix_uniform,
            texture_sampler_uniform,
            mask_sampler_uniform,
            color_uniform
        })
    }
}

