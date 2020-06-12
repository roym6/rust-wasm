use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod cell;
mod fps;
mod universe;
mod utils;

use crate::universe::Universe;
use crate::utils::{cancel_animation_frame, element_by_id, request_animation_frame};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static CELL_SIZE: u32 = 5; // px
static GRID_COLOR: &str = "#CCCCCC";
static DEAD_COLOR: &str = "#FFFFFF";
static ALIVE_COLOR: &str = "#000000";

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    // TODO: implement click handler for cells
    let mut fps = fps::Fps::new();
    let mut universe = Universe::new();

    let height = universe.height();
    let width = universe.width();

    let button = element_by_id("play-pause").dyn_into::<web_sys::HtmlButtonElement>()?;
    let button = Rc::new(button);

    let canvas = element_by_id("game-of-life-canvas").dyn_into::<web_sys::HtmlCanvasElement>()?;
    canvas.set_height((CELL_SIZE + 1) * height + 1);
    canvas.set_width((CELL_SIZE + 1) * width + 1);

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let animation_id = Rc::new(RefCell::new(0));
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    // Main render loop
    {
        let animation_id = animation_id.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            fps.render();
            draw_grid(&context, height, width);
            draw_cells(&context, &universe);

            universe.tick();

            *animation_id.borrow_mut() = request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
    }

    // Handles button click event
    {
        let animation_id = animation_id.clone();
        let g = g.clone();
        let button_copy = button.clone();
        let toggle_play_pause = Closure::wrap(Box::new(move || {
            if *animation_id.borrow() == 0 {
                button_copy.set_inner_text("⏸");
                *animation_id.borrow_mut() = request_animation_frame(g.borrow().as_ref().unwrap());
            } else {
                button_copy.set_inner_text("▶");
                cancel_animation_frame(*animation_id.borrow());
                *animation_id.borrow_mut() = 0;
            }
        }) as Box<dyn FnMut()>);

        button.set_onclick(Some(toggle_play_pause.as_ref().unchecked_ref()));
        toggle_play_pause.forget();
    }

    // Starts simulation
    {
        let animation_id = animation_id.clone();
        button.set_inner_text("⏸");
        *animation_id.borrow_mut() = request_animation_frame(g.borrow().as_ref().unwrap());
    }

    Ok(())
}

fn draw_grid(context: &web_sys::CanvasRenderingContext2d, height: u32, width: u32) {
    context.begin_path();

    let grid_syle = JsValue::from(String::from(GRID_COLOR));
    context.set_stroke_style(&grid_syle);

    for i in 0..=width {
        let x = i * (CELL_SIZE + 1) + 1;
        context.move_to(x as f64, 0f64);
        context.line_to(x as f64, ((CELL_SIZE + 1) * height + 1) as f64);
    }

    for i in 0..=height {
        let y = i * (CELL_SIZE + 1) + 1;
        context.move_to(0f64, y as f64);
        context.line_to(((CELL_SIZE + 1) * width + 1) as f64, y as f64);
    }

    context.stroke();
}

fn draw_cells(context: &web_sys::CanvasRenderingContext2d, universe: &Universe) {
    let height = universe.height();
    let width = universe.width();
    let cells = universe.cells();

    context.begin_path();

    let alive_style = JsValue::from(String::from(ALIVE_COLOR));
    context.set_fill_style(&alive_style);
    for row in 0..height {
        for col in 0..width {
            let idx = universe.get_index(row, col);
            if cells[idx] != cell::Cell::Alive {
                continue;
            }

            context.fill_rect(
                (col * (CELL_SIZE + 1) + 1) as f64,
                (row * (CELL_SIZE + 1) + 1) as f64,
                CELL_SIZE as f64,
                CELL_SIZE as f64,
            );
        }
    }

    let dead_style = JsValue::from(String::from(DEAD_COLOR));
    context.set_fill_style(&dead_style);
    for row in 0..height {
        for col in 0..width {
            let idx = universe.get_index(row, col);
            if cells[idx] != cell::Cell::Dead {
                continue;
            }

            context.fill_rect(
                (col * (CELL_SIZE + 1) + 1) as f64,
                (row * (CELL_SIZE + 1) + 1) as f64,
                CELL_SIZE as f64,
                CELL_SIZE as f64,
            );
        }
    }

    context.stroke();
}
