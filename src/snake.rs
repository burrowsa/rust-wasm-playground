use snake::{game_step, Coordinate, Game, GameDisplay, GameInput, SnakeChange, Turn};
use web_sys::CanvasRenderingContext2d;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::utils::set_panic_hook;

#[wasm_bindgen]
pub struct Snake {
    game: Rc<RefCell<Game>>,
    counter: Rc<RefCell<u32>>,
    input: Rc<WebInput>,
}

struct CanvasDisplay {
    context: Rc<CanvasRenderingContext2d>,
    width: u32,
    height: u32,
}

impl CanvasDisplay {
    fn new(context: CanvasRenderingContext2d, width: u32, height: u32) -> CanvasDisplay {
        CanvasDisplay {
            context: Rc::new(context),
            width,
            height,
        }
    }

    fn draw(&self, game: &Game) {
        let cell_w: f64 = f64::from(self.width - 10) / f64::from(game.width);
        let cell_h: f64 = f64::from(self.height - 10) / f64::from(game.height);

        if game.game_over {
            self.context.set_fill_style(&"#FFCCCC".into());
        } else {
            self.context.set_fill_style(&"#FFFFFF".into());
        }
        self.context.fill_rect(
            5.0,
            5.0,
            (self.width - 10).into(),
            (self.height - 10).into(),
        );

        if game.game_over {
            self.context.set_fill_style(&"#FF6666".into());
        } else {
            self.context.set_fill_style(&"#66FF66".into());
        }

        if let Some(Coordinate { x, y }) = game.snake.front() {
            self.context.fill_rect(
                5.0 + f64::from(*x) * cell_w,
                5.0 + f64::from(*y) * cell_h,
                cell_w,
                cell_h,
            );
        }

        if game.game_over {
            self.context.set_fill_style(&"#FF0000".into());
        } else {
            self.context.set_fill_style(&"#00FF00".into());
        }

        for Coordinate { x, y } in game.snake.iter().skip(1) {
            self.context.fill_rect(
                5.0 + f64::from(*x) * cell_w,
                5.0 + f64::from(*y) * cell_h,
                cell_w,
                cell_h,
            );
        }

        if game.game_over {
            self.context.set_stroke_style(&"#FF0000".into());
        } else{
            self.context.set_stroke_style(&"#000000".into());
        }
        self.context.stroke_rect(
            5.0,
            5.0,
            (self.width - 10).into(),
            (self.height - 10).into(),
        );
    }
}

impl GameDisplay for CanvasDisplay {
    fn game_over(&self, game: &Game) {
        self.draw(game)
    }

    fn initialize(&self, game: &Game) {
        self.draw(game)
    }

    fn update(&self, game: &Game, _change: &SnakeChange) {
        self.draw(game)
    }
}

struct WebInput {
    keypresses: RefCell<VecDeque<Turn>>,
}

impl WebInput {
    fn new() -> WebInput {
        WebInput {
            keypresses: RefCell::new(VecDeque::new()),
        }
    }

    fn push_keypress(&self, turn: Turn) {
        self.keypresses.borrow_mut().push_back(turn);
    }
}

impl GameInput for WebInput {
    fn poll(&self) -> Option<Turn> {
        self.keypresses.borrow_mut().pop_front()
    }
}

#[wasm_bindgen]
impl Snake {
    pub fn new(id: &str, width: u16, height: u16) -> Snake {
        set_panic_hook();

        let result = Snake {
            game: Rc::new(RefCell::new(Game::new(&width, &height))),
            counter: Rc::new(RefCell::new(0)),
            input: Rc::new(WebInput::new()),
        };

        result.start(id);

        result
    }

    fn start(&self, id: &str) {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(id).unwrap();

        {
            let input_ref = self.input.clone();

            let on_keydown: Closure<dyn FnMut(_)> =
                Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                    let key = event.key();
                    if key == "ArrowLeft" {
                        input_ref.push_keypress(Turn::Left);
                        event.prevent_default();
                    } else if key == "ArrowRight" {
                        input_ref.push_keypress(Turn::Right);
                        event.prevent_default();
                    }
                }));

            document
                .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
                .expect("failed setting keydown handler");

            on_keydown.forget();
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

        let display = CanvasDisplay::new(context, canvas.width(), canvas.height());

        display.initialize(&*self.game.borrow_mut());

        #[allow(clippy::type_complexity)]
        let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let outer_f = f.clone();
        {
            let game_ref = self.game.clone();
            let counter_ref = self.counter.clone();
            let input_ref = self.input.clone();
            let mut frames: u32 = 0;

            *outer_f.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                let mut counter = counter_ref.borrow_mut();
                let mut game = game_ref.borrow_mut();

                if frames >= 10 {
                    frames = 0;
                    game_step(&mut counter, &mut game, &display, &*input_ref);
                } else {
                    frames += 1;
                }

                if !game.game_over {
                    web_sys::window()
                        .unwrap()
                        .request_animation_frame(
                            f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                        )
                        .expect("failed requesting animation frame");
                }
            })));
        }

        web_sys::window()
            .unwrap()
            .request_animation_frame(outer_f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("failed requesting animation frame");
    }
}
