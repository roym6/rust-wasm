use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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

// TODO: these really should return Results instead of causing panics
pub fn window() -> web_sys::Window {
    web_sys::window().expect("No global `window` exists")
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("Should have `document` on `window`")
}

pub fn element_by_id(id: &str) -> web_sys::Element {
    document()
        .get_element_by_id(id)
        .expect(format!("Should have {} on `document`", id).as_str())
}

pub fn html_element_by_id(id: &str) -> web_sys::HtmlElement {
    element_by_id(id)
        .dyn_into::<web_sys::HtmlElement>()
        .expect("Should have `HtmlElement` on `document`")
}

pub fn performance() -> web_sys::Performance {
    window()
        .performance()
        .expect("Should have `performance` on `window`")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) -> i32 {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("Should register `requestAnimationFrame`")
}

pub fn cancel_animation_frame(handle: i32) {
    window()
        .cancel_animation_frame(handle)
        .expect("Should cancel animation frame");
}
