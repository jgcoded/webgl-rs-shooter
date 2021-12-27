use std::convert::TryInto;

use js_sys::Float32Array;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{WebGl2RenderingContext, HtmlCanvasElement};

use crate::{vector::Vec3, matrix::Mat4};


pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn window() -> web_sys::Window {
    web_sys::window().unwrap()
}

pub fn document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

pub fn get_canvas(canvas_id: &str) -> Result<HtmlCanvasElement, JsValue> {
    let canvas = document().get_element_by_id(canvas_id).unwrap();
    let canvas = canvas.dyn_into::<HtmlCanvasElement>()?;

    Ok(canvas)
}

pub fn get_rendering_context(canvas: &HtmlCanvasElement) -> Result<WebGl2RenderingContext, JsValue> {
    
    // set per MDN docs
    canvas.set_width(canvas.client_width().try_into().unwrap());
    canvas.set_height(canvas.client_height().try_into().unwrap());

    let gl =
        canvas.get_context("webgl2")?
        .unwrap()
        .dyn_into::<web_sys::WebGl2RenderingContext>()?;

    /*
        When you first create a WebGL context,
        the size of the viewport will match
        the size of the canvas. However,
        if you resize the canvas, you will
        need to tell the WebGL context a new
        viewport setting. In this situation,
        you can use gl.viewport. 
    */
    gl.viewport(0, 0, gl.drawing_buffer_width(), gl.drawing_buffer_height());
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    Ok(gl)
}

pub fn request_animation_frame(f: &Closure<dyn FnMut(&JsValue)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

impl Vec3 {
    pub fn to_js(&self) -> Float32Array {
        let data = Float32Array::new_with_length(3);
        data.copy_from(self.data());
        data
    }
}

impl Mat4 {
    pub fn to_js(&self) -> Float32Array {
        let data = Float32Array::new_with_length(3);
        data.copy_from(self.data());
        data
    }
}