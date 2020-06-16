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

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

static CELL_SIZE: u32 = 5; // px
static GRID_COLOR: &str = "#CCCCCC";
static DEAD_COLOR: &str = "#FFFFFF";
static ALIVE_COLOR: &str = "#000000";

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    let mut fps = fps::Fps::new();
    let universe = Rc::new(RefCell::new(Universe::new()));

    let button = element_by_id("play-pause").dyn_into::<web_sys::HtmlButtonElement>()?;
    let button = Rc::new(button);

    let canvas_elem = element_by_id("game-of-life-canvas");
    let canvas = canvas_elem
        .clone()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let height = universe.borrow().height();
    let width = universe.borrow().width();
    canvas.set_height((CELL_SIZE + 1) * height + 1);
    canvas.set_width((CELL_SIZE + 1) * width + 1);

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
    let context = Rc::new(context);

    add_clear_handler(Rc::clone(&context), Rc::clone(&universe));

    let animation_id = Rc::new(RefCell::new(0));
    let recursive_render_loop = Rc::new(RefCell::new(None));
    let outer_render_loop = Rc::clone(&recursive_render_loop);

    // Main render loop
    {
        let animation_id = Rc::clone(&animation_id);
        let universe = Rc::clone(&universe);
        let context = Rc::clone(&context);

        *outer_render_loop.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            fps.render();
            draw_grid(&context, &*universe.borrow());
            draw_cells(&context, &*universe.borrow());

            universe.borrow_mut().tick();

            *animation_id.borrow_mut() =
                request_animation_frame(recursive_render_loop.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
    }

    // Handles button click event
    {
        let animation_id = Rc::clone(&animation_id);
        let outer_render_loop = Rc::clone(&outer_render_loop);
        let button_copy = Rc::clone(&button);

        let toggle_play_pause = Closure::wrap(Box::new(move || {
            if *animation_id.borrow() == 0 {
                button_copy.set_inner_text("⏸");
                *animation_id.borrow_mut() =
                    request_animation_frame(outer_render_loop.borrow().as_ref().unwrap());
            } else {
                button_copy.set_inner_text("▶");
                cancel_animation_frame(*animation_id.borrow());
                *animation_id.borrow_mut() = 0;
            }
        }) as Box<dyn FnMut()>);

        button.set_onclick(Some(toggle_play_pause.as_ref().unchecked_ref()));
        toggle_play_pause.forget();
    }

    // Handles cell clicking
    {
        let universe = Rc::clone(&universe);
        let canvas_copy = canvas.clone();

        let cell_click_handler = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let bounding_rect = canvas_elem.get_bounding_client_rect();
            let scale_x = canvas_copy.width() as f64 / bounding_rect.width();
            let scale_y = canvas_copy.height() as f64 / bounding_rect.height();

            let canvas_left = (event.client_x() as f64 - bounding_rect.left()) * scale_x;
            let canvas_top = (event.client_y() as f64 - bounding_rect.top()) * scale_y;

            let row = (canvas_top / (CELL_SIZE as f64 + 1f64))
                .floor()
                .min(height as f64 - 1f64) as u32;
            let col = (canvas_left / (CELL_SIZE as f64 + 1f64))
                .floor()
                .min(width as f64 - 1f64) as u32;

            universe.borrow_mut().toggle_cell(row, col);
            draw_cells(&context, &*universe.borrow());
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback(
            "click",
            cell_click_handler.as_ref().unchecked_ref(),
        )?;
        cell_click_handler.forget();
    }

    // Starts simulation
    {
        let animation_id = Rc::clone(&animation_id);
        button.set_inner_text("⏸");
        *animation_id.borrow_mut() =
            request_animation_frame(outer_render_loop.borrow().as_ref().unwrap());
    }

    Ok(())
}

fn add_clear_handler(
    context: Rc<web_sys::CanvasRenderingContext2d>,
    universe: Rc<RefCell<Universe>>,
) {
    let button = element_by_id("clear")
        .dyn_into::<web_sys::HtmlButtonElement>()
        .unwrap();
    let clear_handler = Closure::wrap(Box::new(move || {
        universe.borrow_mut().clear();
        draw_cells(&context, &*universe.borrow());
    }) as Box<dyn FnMut()>);

    button.set_onclick(Some(clear_handler.as_ref().unchecked_ref()));
    clear_handler.forget();
}

fn draw_grid(context: &web_sys::CanvasRenderingContext2d, universe: &Universe) {
    let height = universe.height();
    let width = universe.width();

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
