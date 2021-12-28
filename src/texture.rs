use std::convert::TryInto;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;
use web_sys::{WebGl2RenderingContext, HtmlImageElement, WebGlTexture};


// texture loading based off of
// https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial/Using_textures_in_WebGL
pub fn load_texture(
    gl: &WebGl2RenderingContext,
    source: &str
) -> Result<Rc<WebGlTexture>, JsValue> {

    let texture = gl.create_texture()
        .ok_or_else(|| String::from("Could not make new webgl texture"))?;

    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

    let level = 0;
    let internal_format = WebGl2RenderingContext::RGBA as i32;
    let width = 1;
    let height = 1;
    let border = 0;
    let src_format = WebGl2RenderingContext::RGBA;
    let src_type = WebGl2RenderingContext::UNSIGNED_BYTE;
    let src_data = [0u8, 0u8, 255u8, 255u8];  // opaque blue

    //https://docs.rs/web-sys/latest/web_sys/struct.WebGl2RenderingContext.html#method.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        WebGl2RenderingContext::TEXTURE_2D,
        level,
        internal_format,
        width,
        height,
        border,
        src_format,
        src_type,
        Some(&src_data)
    )?;

    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

    let image = HtmlImageElement::new()?;
    let image_rc = Rc::new(image);
    let texture_rc = Rc::new(texture);

    {
        let image = image_rc.clone();
        let texture = texture_rc.clone();
        let gl = Rc::new(gl.clone());

        let on_load_callback = Closure::wrap(Box::new(move || {
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

            let result = gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
                WebGl2RenderingContext::TEXTURE_2D,
                level,
                internal_format,
                src_format,
                src_type,
                &image
            );

            match result {
                Err(e) => {
                    web_sys::console::log_2(&"load_texture".into(), &e);
                    return;
                },
                _ => ()
            };

            gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        }) as Box<dyn FnMut()>);

        image_rc.set_onload(Some(on_load_callback.as_ref().unchecked_ref()));
        on_load_callback.forget();
    }

    image_rc.set_src(source);

    Ok(texture_rc)
}
