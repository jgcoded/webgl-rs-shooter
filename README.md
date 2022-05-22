# WebGL Rust 2D Shooter

Porting the awesome Riemer's 2D shooter game to Rust with WASM and WebGL for fun. Work in progress.

## Setup

Install wasm-pack
https://rustwasm.github.io/wasm-pack/installer/

Install npm

At the root of the repo:
```
$ npm install
```

## Build

```
$ wasm-pack build && npm run serve
```

Then visit http://localhost:8080/ on your browser.

## References

### Riemer's XNA Game Tutorial

https://github.com/simondarksidej/XNAGameStudio/wiki/Riemers2DXNAoverview


### Rust

https://doc.rust-lang.org/stable/book/
https://doc.rust-lang.org/stable/rust-by-example/index.html

### Web Assembly

https://rustwasm.github.io/docs.html
https://rustwasm.github.io/book/introduction.html
https://docs.rs/web-sys/latest/web_sys/
https://docs.rs/js-sys/latest/js_sys/
https://rustwasm.github.io/wasm-bindgen/introduction.html
https://rustwasm.github.io/wasm-bindgen/examples/paint.html
https://rustwasm.github.io/wasm-bindgen/examples/request-animation-frame.html

### OpenGL and WebGL

https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Tutorial/Using_textures_in_WebGL
https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/code#handle_keyboard_events_in_a_game
https://learnopengl.com/In-Practice/2D-Game/Rendering-Sprites
https://github.com/WebGLSamples/WebGL2Samples
https://www.khronos.org/files/webgl20-reference-guide.pdf

### Game Architecture

https://gameprogrammingpatterns.com/contents.html

### Potential Game Art

To spice things up.

https://opengameart.org/content/animated-tank
https://opengameart.org/content/tileset-and-assets-for-a-scorched-earth-type-game
https://opengameart.org/content/backgrounds-for-2d-platformers
http://google.github.io/tracing-framework/