use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Fps {
    element: web_sys::Element,
    frames: Vec<f64>,
    last_time_stamp: f64,
}

#[wasm_bindgen]
impl Fps {
    pub fn render(&mut self) {
        // TODO: avg and max initially show as `inf`, why?
        let now = web_sys::window().unwrap().performance().unwrap().now();
        let delta = now - self.last_time_stamp;
        self.last_time_stamp = now;
        let fps = (1f64 / delta) * 1000f64;

        self.frames.push(fps);
        if self.frames.len() > 100 {
            self.frames.remove(0);
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;
        let mut sum = 0f64;
        for frame in self.frames.iter() {
            sum += frame;
            min = min.min(*frame);
            max = max.max(*frame);
        }
        let mean = sum / (self.frames.len() as f64);
        self.element.set_text_content(Some(
            format!(
                "
            Frames per Second:
                     latest = {}
            avg of last 100 = {}
            min of last 100 = {}
            max of last 100 = {}
        ",
                fps.round(),
                mean.round(),
                min.round(),
                max.round(),
            )
            .as_str(),
        ));
    }
    pub fn new() -> Option<Fps> {
        let window = web_sys::window()?;
        let element = window.document()?.get_element_by_id("fps")?;
        let performance = window.performance()?;

        Some(Fps {
            element,
            frames: vec![],
            last_time_stamp: performance.now(),
        })
    }
}
