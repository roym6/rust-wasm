use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod cell;
mod fps;
mod universe;
mod utils;

use crate::universe::Universe;
use crate::utils::{cancel_animation_frame, element_by_id, request_animation_frame, window};

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

static CELL_SIZE: u32 = 10; // px
static GRID_COLOR: &str = "#CCCCCC";
static DEAD_COLOR: &str = "#FFFFFF";
static ALIVE_COLOR: &str = "#000000";
static HOVER_COLOR: &str = "#FF5500";

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
        let canvas_elem = canvas_elem.clone();
        let canvas_copy = canvas.clone();
        let context = Rc::clone(&context);

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

    add_drag_handlers(
        canvas_elem.clone(),
        Rc::clone(&context),
        Rc::clone(&universe),
        Rc::clone(&animation_id),
    );

    // Starts simulation
    {
        let animation_id = Rc::clone(&animation_id);
        button.set_inner_text("⏸");
        *animation_id.borrow_mut() =
            request_animation_frame(outer_render_loop.borrow().as_ref().unwrap());
    }

    Ok(())
}

fn add_drag_handlers(
    canvas_elem: web_sys::Element,
    context: Rc<web_sys::CanvasRenderingContext2d>,
    universe: Rc<RefCell<Universe>>,
    animation_id: Rc<RefCell<i32>>,
) {
    let window = window();
    let canvas_html_elem = canvas_elem
        .clone()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();

    let canvas = canvas_elem
        .clone()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    let height = universe.borrow().height();
    let width = universe.borrow().width();
    let prefab_universe: Rc<RefCell<Option<Universe>>> = Rc::new(RefCell::new(None));

    {
        let prefab_universe = Rc::clone(&prefab_universe);
        let drag_start_handler = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
            if *animation_id.borrow() != 0 {
                // TODO: update button text
                cancel_animation_frame(*animation_id.borrow());
                *animation_id.borrow_mut() = 0;
                element_by_id("play-pause").dyn_into::<web_sys::HtmlButtonElement>().unwrap().set_inner_text("▶");
            }
            let src_id = event
                .data_transfer()
                .unwrap()
                .get_data("text/plain")
                .unwrap();
            let elem = utils::html_element_by_id(src_id.as_str());
            *prefab_universe.borrow_mut() = Some(Universe::from(elem));
        }) as Box<dyn FnMut(_)>);
        // TODO: can this be canvas instead of window?
        window.set_ondragstart(Some(drag_start_handler.as_ref().unchecked_ref()));
        drag_start_handler.forget();
    }

    let painted_cells: Rc<RefCell<Vec<(u32, u32)>>> = Rc::new(RefCell::new(vec![]));

    {
        let context = Rc::clone(&context);
        let universe = Rc::clone(&universe);
        let prefab_universe = Rc::clone(&prefab_universe);
        let painted_cells = Rc::clone(&painted_cells);

        // TODO: needs edge logic for shapes that wrap around world on borders
        let drag_over_handler = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
            event.dyn_ref::<web_sys::Event>().unwrap().prevent_default();
            reset_cells(&context, &universe.borrow(), &painted_cells.borrow());
            painted_cells.borrow_mut().clear();

            let bounding_rect = canvas_elem.get_bounding_client_rect();
            let scale_x = canvas.width() as f64 / bounding_rect.width();
            let scale_y = canvas.height() as f64 / bounding_rect.height();

            let canvas_left = (event.client_x() as f64 - bounding_rect.left()) * scale_x;
            let canvas_top = (event.client_y() as f64 - bounding_rect.top()) * scale_y;

            let prefab_height = prefab_universe.borrow().as_ref().unwrap().height();
            let prefab_width = prefab_universe.borrow().as_ref().unwrap().width();

            let row = (canvas_top / (CELL_SIZE + 1) as f64) - ((prefab_height / 2) as f64).floor();
            let row = if prefab_height % 2 == 1 {
                row.floor()
            } else {
                row.round()
            };
            let row = row.min(height as f64 - 1f64) as u32;
            let col = (canvas_left / (CELL_SIZE + 1) as f64) - ((prefab_width / 2) as f64).floor();
            let col = if prefab_width % 2 == 1 {
                col.floor()
            } else {
                col.round()
            };
            let col = col.min(width as f64 - 1f64) as u32;

            let hover_style = JsValue::from(String::from(HOVER_COLOR));
            context.set_fill_style(&hover_style);
            for prefab_row in 0..prefab_height {
                for prefab_col in 0..prefab_width {
                    let idx = prefab_universe
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .get_index(prefab_row, prefab_col);
                    if prefab_universe.borrow().as_ref().unwrap().cells()[idx] == cell::Cell::Alive
                    {
                        context.fill_rect(
                            ((col + prefab_col) * (CELL_SIZE + 1) + 1) as f64,
                            ((row + prefab_row) * (CELL_SIZE + 1) + 1) as f64,
                            CELL_SIZE as f64,
                            CELL_SIZE as f64,
                        );
                        painted_cells
                            .borrow_mut()
                            .push((row + prefab_row, col + prefab_col));
                    }
                }
            }

            // TODO: need to return false?
            // false
        }) as Box<dyn FnMut(_)>);
        canvas_html_elem.set_ondragover(Some(drag_over_handler.as_ref().unchecked_ref()));
        drag_over_handler.forget();
    }

    let drop_handler = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
        event.dyn_ref::<web_sys::Event>().unwrap().prevent_default();
        event
            .dyn_ref::<web_sys::Event>()
            .unwrap()
            .stop_propagation();

        for (row, col) in painted_cells.borrow().iter() {
            // TODO: this shouldn't be toggle
            universe.borrow_mut().toggle_cell(*row, *col);
        }
        // TODO: rename func
        reset_cells(&context, &universe.borrow(), &painted_cells.borrow());
        painted_cells.borrow_mut().clear();
        *prefab_universe.borrow_mut() = None;
    }) as Box<dyn FnMut(_)>);
    canvas_html_elem.set_ondrop(Some(drop_handler.as_ref().unchecked_ref()));
    drop_handler.forget();

    // TODO: add drag end handler
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

// TODO: this is very similar to draw_cells, maybe refactor into one?
fn reset_cells(
    context: &web_sys::CanvasRenderingContext2d,
    universe: &Universe,
    cells_to_reset: &Vec<(u32, u32)>,
) {
    let alive_style = JsValue::from(String::from(ALIVE_COLOR));
    context.set_fill_style(&alive_style);
    for (row, col) in cells_to_reset.clone().iter() {
        // TODO: needs edge logic for shapes that wrap around world on borders
        if *row >= universe.height() || *col >= universe.width() {
            continue;
        }

        let idx = universe.get_index(*row, *col);
        if universe.cells()[idx] != cell::Cell::Alive {
            continue;
        }

        context.fill_rect(
            (col * (CELL_SIZE + 1) + 1) as f64,
            (row * (CELL_SIZE + 1) + 1) as f64,
            CELL_SIZE as f64,
            CELL_SIZE as f64,
        );
    }

    let dead_style = JsValue::from(String::from(DEAD_COLOR));
    context.set_fill_style(&dead_style);
    for (row, col) in cells_to_reset.clone().iter() {
        // TODO: needs edge logic for shapes that wrap around world on borders
        if *row >= universe.height() || *col >= universe.width() {
            continue;
        }
        let idx = universe.get_index(*row, *col);
        if universe.cells()[idx] != cell::Cell::Dead {
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
