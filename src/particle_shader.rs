use wasm_bindgen::JsValue;
use web_sys::{WebGlProgram, WebGlUniformLocation, WebGl2RenderingContext};

use crate::shader::new_shader_program;



pub struct ParticleShader {
    pub program: WebGlProgram,
    pub vertex_position_attrib: i32,
    pub vertex_texture_attrib: i32,
    pub projection_matrix_uniform: WebGlUniformLocation,
    pub texture_sampler_uniform: WebGlUniformLocation,
    pub color_uniform: WebGlUniformLocation,
    pub offset_uniform: WebGlUniformLocation,
    pub scale_uniform: WebGlUniformLocation
}

impl ParticleShader {

    pub fn new(gl: &WebGl2RenderingContext) -> Result<ParticleShader, JsValue> {
        let vertex_shader_src = r##"
        attribute vec4 aVertexPosition;
        attribute vec2 aTextureCoord;

        uniform mat4 uProjectionMatrix;
        uniform float uScale;
        uniform vec3 uOffset;

        varying highp vec2 vTextureCoord;

        void main(void) {
          gl_Position = uProjectionMatrix * vec4((aVertexPosition.xy * uScale) + uOffset.xy, 0.0, 1.0);
          vTextureCoord = aTextureCoord;
        }
            "##;

        let fragment_shader_src = r##"
        precision mediump float;

        varying highp vec2 vTextureCoord;

        uniform sampler2D uTextureSampler;
        uniform vec4 uColor;

        void main(void) {
            vec4 texture = texture2D(uTextureSampler, vTextureCoord);
            gl_FragColor = texture * uColor;
        }
            "##;

        let program = new_shader_program(gl, vertex_shader_src, fragment_shader_src)?;

        let vertex_position_attrib = gl.get_attrib_location(&program, "aVertexPosition");
        let vertex_texture_attrib = gl.get_attrib_location(&program, "aTextureCoord");

        let projection_matrix_uniform = gl
            .get_uniform_location(&program, "uProjectionMatrix")
            .ok_or_else(|| String::from("Could not get projection uniform location"))?;
        let texture_sampler_uniform = gl
            .get_uniform_location(&program, "uTextureSampler")
            .ok_or_else(|| String::from("Could not get texture sampler uniform location"))?;

        let color_uniform = gl
            .get_uniform_location(&program, "uColor")
            .ok_or_else(|| String::from("Could not get color sampler uniform location"))?;

        let scale_uniform = gl
            .get_uniform_location(&program, "uScale")
            .ok_or_else(|| String::from("Could not get scale uniform location"))?;

        let offset_uniform = gl
            .get_uniform_location(&program, "uOffset")
            .ok_or_else(|| String::from("Could not get offset uniform location"))?;

        Ok(ParticleShader {
            program,
            vertex_position_attrib,
            vertex_texture_attrib,
            projection_matrix_uniform,
            texture_sampler_uniform,
            color_uniform,
            scale_uniform,
            offset_uniform
        })
    }

}

