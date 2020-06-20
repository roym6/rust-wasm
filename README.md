A Rust/wasm implementation of the <a href="https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life">Game of life</a>. This project is based on this <a href="https://rustwasm.github.io/docs/book/">tutorial</a>, but handles all rendering and user interactions with Rust instead of JS.

To build the project, you will need <a href="https://rustwasm.github.io/wasm-pack/installer/">wasm-pack</a>.
1. In the root folder run `wasm-pack build`.
2. In the `www` folder run `npm run start` to start the webpack dev server.
3. Access the app at localhost:8080

You can use <a href="https://github.com/passcod/cargo-watch">cargo watch</a> to auto build the wasm package using `cargo-watch -i www -s "wasm-pack build"`.

Dockerfile also included to build/launch the project.
TCS hosted app available at http://wasm-game-of-life.tcs.techempower.com

<h2>Background</h2>
<a href="https://developer.mozilla.org/en-US/docs/WebAssembly">WebAssembly</a> is an assembly-like language that runs in the browser, offering near-native performance. It includes a JS API allowing JS interoperability, with plans to add direct access access to the broswer rendering engine to provide faster-than-JS DOM manipulation (<a href="https://hacks.mozilla.org/2019/08/webassembly-interface-types/">in-depth article on the topic</a>). Technically, any language can be compiled into wasm, but Rust provides C/C++ levels of performance along with memory safety and has very mature wasm tooling, making it a breeze to start a new app.

<h2>Code Explanations</h2>
The tutorial linked above, along with the <a href="https://rustwasm.github.io/docs/wasm-bindgen/introduction.html">wasm-bindgen guide</a>, cover all of the concepts used in this tutorial, so I'll only be giving a brief explanation of some relevant code for those interested in a quick run through.

First off, <a href="https://github.com/rustwasm/wasm-bindgen">wasm-bindgen</a>. This crate provides the meat of the functionality, giving us the ability to "import JavaScript things into Rust and export Rust things to JavaScript". Use it by adding the attribute to any function/struct you want to import/export:
``` rust
use wasm_bindgen::prelude::*;

// Import the `window.alert` function from the Web.
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// Export a `greet` function from Rust to JavaScript, that alerts a
// hello message.
#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

// Export an Object
#[wasm_bindgen]
pub struct Fps {
  ...
}

// No wasm_bindgen attribute, non-public methods, not exported to JS
impl Fps {
  fn private_method() {
    ...
  }
}

#[wasm_bindgen]
impl Fps {
  pub fn render(&mut self) {
    ...
  }
  pub fn new() -> Fps {
    ...
  }
}
```
Use the exported code in JS:
``` javascript
import { greet, Fps } from "./hello_world";

greet("World!");

const fps = Fps.new();
fps.render();
// Can't do
fps.private_method();
```

<h2>Side notes</h2>

- Wasm currently uses 32bit pointers, restricted to 4GB of memory. 64bit support planned.
- Dom manipulation through wasm is currently shimmed using JS, so there is no real benefit to handling UI related features with wasm until Web IDL bindings are available. This project was more intended as a proof of concept of what can be done.
