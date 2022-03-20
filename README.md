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

# References


https://rustwasm.github.io/docs.html
https://rustwasm.github.io/book/introduction.html

https://docs.rs/web-sys/latest/web_sys/
https://docs.rs/js-sys/latest/js_sys/

https://rustwasm.github.io/wasm-bindgen/introduction.html
https://rustwasm.github.io/wasm-bindgen/examples/paint.html

https://github.com/simondarksidej/XNAGameStudio/wiki/Riemers2DXNAoverview

https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/code#handle_keyboard_events_in_a_game
