use rand::seq::SliceRandom;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::utils::set_panic_hook;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct Rectangle(f64, f64, f64, f64);

#[wasm_bindgen]
pub struct Smiley {
    x: Rc<RefCell<f64>>,
    y: Rc<RefCell<f64>>,
    clear: Rc<RefCell<Option<Rectangle>>>,
}

#[wasm_bindgen]
impl Smiley {
    pub fn new(id: &str) -> Smiley {
        set_panic_hook();
        
        let result = Smiley {
            x: Rc::new(RefCell::new(0.0)),
            y: Rc::new(RefCell::new(0.0)),
            clear: Rc::new(RefCell::new(None)),
        };

        result.start(id);

        result
    }

    fn start(&self, id: &str) {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(id).unwrap();

        {
            let x_ref = self.x.clone();
            let y_ref = self.y.clone();
            let clear_ref = self.clear.clone();

            let on_mousemove: Closure<dyn FnMut(_)> =
                Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                    let x = f64::from(event.offset_x());
                    let y = f64::from(event.offset_y());
                    // web_sys::console::log_3(
                    //     &"click (%s, %s)".into(),
                    //     &x.into(),
                    //     &y.into(),
                    // );

                    let old_x = *x_ref.borrow();
                    let old_y = *y_ref.borrow();
                    *x_ref.borrow_mut() = x - 75.0;
                    *y_ref.borrow_mut() = y - 75.0;
                    let mut clear = clear_ref.borrow_mut();
                    if clear.is_none() {
                        *clear = Some(Rectangle(old_x, old_y, 150.0, 150.0));
                    }
                }));

            canvas
                .add_event_listener_with_callback(
                    "mousemove",
                    on_mousemove.as_ref().unchecked_ref(),
                )
                .expect("failed setting onmousemove handler");

            on_mousemove.forget();
        }

        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        #[allow(clippy::type_complexity)]
        let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let outer_f = f.clone();
        {
            let x_ref = self.x.clone();
            let y_ref = self.y.clone();
            let clear_ref = self.clear.clone();

            *outer_f.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                let x = *x_ref.borrow();
                let y = *y_ref.borrow();
                let mut clear = clear_ref.borrow_mut();

                if let Some(Rectangle(cx, cy, wx, wy)) = *clear {
                    context.clear_rect(cx, cy, wx, wy);
                    *clear = None;
                }

                context.begin_path();

                let color: JsValue = vec!["red", "orange", "green", "blue", "yellow", "purple"]
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .to_string()
                    .into();
                context.set_stroke_style(&color);

                // Draw the outer circle.
                context
                    .arc(75.0 + x, 75.0 + y, 50.0, 0.0, f64::consts::PI * 2.0)
                    .unwrap();

                // Draw the mouth.
                context.move_to(110.0 + x, 75.0 + y);
                context
                    .arc(75.0 + x, 75.0 + y, 35.0, 0.0, f64::consts::PI)
                    .unwrap();

                // Draw the left eye.
                context.move_to(65.0 + x, 65.0 + y);
                context
                    .arc(60.0 + x, 65.0 + y, 5.0, 0.0, f64::consts::PI * 2.0)
                    .unwrap();

                // Draw the right eye.
                context.move_to(95.0 + x, 65.0 + y);
                context
                    .arc(90.0 + x, 65.0 + y, 5.0, 0.0, f64::consts::PI * 2.0)
                    .unwrap();

                context.stroke();

                web_sys::window()
                    .unwrap()
                    .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                    .expect("failed requesting animation frame");
            })));
        }

        web_sys::window()
            .unwrap()
            .request_animation_frame(outer_f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("failed requesting animation frame");
    }
}
