A Rust/wasm implementation of the <a href="https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life">Game of life</a>. This project is based on this <a href="https://rustwasm.github.io/docs/book/">tutorial</a>, but handles all rendering and user interactions with Rust instead of JS.

To build the project, you will need <a href="https://rustwasm.github.io/wasm-pack/installer/">wasm-pack</a>.
1. In the root folder run `wasm-pack build` to compile the Rust code into wasm.
2. In the `/www` folder run `npm run start` to start the webpack dev server.
3. Access the app at `localhost:8080`

You can use <a href="https://github.com/passcod/cargo-watch">cargo watch</a> to auto build the wasm package using `cargo-watch -i www -s "wasm-pack build"`.

Dockerfile also included to build/launch the project.
TCS hosted app available at http://wasm-game-of-life.tcs.techempower.com

<h2>Background</h2>
<a href="https://developer.mozilla.org/en-US/docs/WebAssembly">WebAssembly</a> is an assembly-like language that runs in the browser, offering near-native performance. It includes a JS API allowing JS interoperability, with plans to add direct access access to the broswer rendering engine to provide faster-than-JS DOM manipulation (<a href="https://hacks.mozilla.org/2019/08/webassembly-interface-types/">in-depth article on the topic</a>). Technically, any language can be compiled into wasm, but Rust provides C/C++ levels of performance along with memory safety and has very mature wasm tooling, making it a breeze to start a new app.

<h2>Code Explanations</h2>
The tutorial linked above, along with the <a href="https://rustwasm.github.io/docs/wasm-bindgen/introduction.html">wasm-bindgen guide</a>, cover all of the concepts used in this repo, so I'll only be giving a brief explanation of some relevant code for those interested in a quick run through.

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

// Export a struct/object
#[wasm_bindgen]
pub struct Fps {
  ...
}

// And its methods
#[wasm_bindgen]
impl Fps {
  pub fn render(&mut self) {
    ...
  }
  pub fn new() -> Fps {
    ...
  }
}

// No wasm_bindgen attribute, non-public methods, not exported to JS
impl Fps {
  fn private_method() {
    ...
  }
}
```
Use the exported code in JS:
``` javascript
import { greet, Fps } from "./my_rust_wasm_package";

greet("World!");

const fps = Fps.new();
fps.render();
// Can't do this!
// fps.private_method();
```

Using the crate <a href="https://docs.rs/web-sys/0.3.40/web_sys/">web_sys</a>, we can get bindings for the Web API. However, to keep the package size small, we need to individually import any features we plan on using. For example:

Grab a DOM element and interact with it:
``` toml
# cargo.toml
[dependencies.web-sys]
version = "0.3.4"
features = [
  'Window',
  'Document',
  'Element'
]
```
``` rust
// my_rust_file.rs
use wasm_bindgen::prelude::*;

let my_element = web_sys::window()?.document()?.get_element_by_id("my-element")?;
my_element.set_text_content(Some("This is my text."));
```
Grab data associated with element:
``` html
<!-- my_html_file.html -->
<div id="my-element" data-custom="9000">This element has custom data.</div>
```
``` toml
# building on previous cargo.toml example
features = [
  ...
  'HtmlElement',
  'DomStringMap',
]
```
``` rust
// my_rust_file.rs
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

let my_element: web_sys::Element = web_sys::window()?.document()?.get_element_by_id("my-element")?;
// Web API features are scoped by type. We have an Element, but want an HtmlElement to access the dataset.
//  Temporarily cast it.
let dataset: web_sys::DomStringMap = my_element.dyn_ref::<web_sys::HtmlElement>()?.dataset();
let custom_data = dataset.get("custom")?.parse::<u32>()?;
my_element.set_text_content(Some(format!("My custom data is: {}", custom_data)));
```
Add a handler to a button:
``` toml
# building on previous cargo.toml example
features = [
  ...
  'HtmlButtonElement',
]
```
``` rust
// my_rust_file.rs
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

let condition = false;
let button = web_sys::window()?.document()?.get_element_by_id("my-button")?;
// We have no need for the button Element, permanently cast into a HtmlButtonElement
let button = button_elem.dyn_into::<web_sys::HtmlButtonElement>()?;
// Will need multiple references to button since our closure requires us to move a reference out of the current scope
let button = Rc::new(button);
let button_copy = Rc::clone(&button);
let toggle_button_text = Closure::wrap(Box::new(move || {
  if condition {
    button.set_inner_text("State 1");
  } else {
    button.set_inner_text("State 2");
  }
}) as Box<dyn FnMut()>);

button_copy.set_onclick(Some(toggle_button_text.as_ref().unchecked_ref()));
toggle_button_text.forget();
```
<h2>Notes</h2>

- Can enable debug symbols when debugging in browser using `wasm-pack build --debug`. Otherwise, stack traces will be a bunch of random strings.
- Wasm currently uses 32bit pointers, so apps are restricted to 4GB of memory. 64bit support is planned.
- DOM manipulation through wasm is currently shimmed using JS, so there is no real benefit to handling UI related features with wasm until Web IDL bindings are available. This project was more intended as a proof of concept of what can be done.
- Don't need a bundler, but it really simplifies your life. Can also use Parcel if you want a light weight bundler (<a href="https://github.com/rustwasm/rust-parcel-template">example template</a>).
