use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    execute,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use snake::{game_step, Coordinate, Game, GameDisplay, GameInput, SnakeChange, Turn};
use std::{io::stdout, thread::sleep, time::Duration};

struct Console {}

impl Console {
    fn board(f: Color, b: Color, w: usize, h: usize) {
        execute!(
            stdout(),
            Clear(ClearType::FromCursorUp),
            MoveTo(0, 0),
            SetBackgroundColor(b),
            SetForegroundColor(f),
            Print(format!("┏{}┓\n", "━".repeat(w))),
            Print(format!("┃{}┃\n", " ".repeat(w)).repeat(h)),
            Print(format!("┗{}┛\n", "━".repeat(w))),
            MoveTo((h + 1).try_into().unwrap(), 0)
        )
        .expect("Problem drawing board");
    }

    fn score(score: usize, f: Color, b: Color, _w: usize, h: usize) {
        execute!(
            stdout(),
            SetBackgroundColor(b),
            SetForegroundColor(f),
            MoveTo(0, (h + 2).try_into().unwrap()),
            Print(format!("Score: {}\n", score))
        )
        .expect("Problem writing out score");
    }

    fn snake_tail(f: Color, b: Color, x: &i32, y: &i32) {
        execute!(
            stdout(),
            MoveTo((x + 1).try_into().unwrap(), (y + 1).try_into().unwrap()),
            SetBackgroundColor(b),
            SetForegroundColor(f),
            Print("/"),
        )
        .expect("Problem drawing snake");
    }

    fn snake_head(f: Color, b: Color, x: &i32, y: &i32) {
        execute!(
            stdout(),
            MoveTo((x + 1).try_into().unwrap(), (y + 1).try_into().unwrap()),
            SetBackgroundColor(b),
            SetForegroundColor(f),
            Print(":"),
        )
        .expect("Problem drawing snake");
    }
}

impl GameDisplay for Console {
    fn initialize(&self, game: &Game) {
        Console::board(
            Color::White,
            Color::Black,
            game.width.into(),
            game.height.into(),
        );

        self.update(
            game,
            &SnakeChange {
                removed: None,
                added: Some(game.snake.front().unwrap().clone()),
            },
        );

        Console::score(
            game.snake.len(),
            Color::White,
            Color::Black,
            game.width.into(),
            game.height.into(),
        );
    }

    fn game_over(&self, game: &Game) {
        Console::board(
            Color::Red,
            Color::Black,
            game.width.into(),
            game.height.into(),
        );

        for Coordinate { x, y } in &game.snake {
            Console::snake_tail(Color::DarkRed, Color::Red, x, y);
        }

        Console::score(
            game.snake.len() + 1,
            Color::Red,
            Color::Black,
            game.width.into(),
            game.height.into(),
        );

        execute!(
            stdout(),
            SetBackgroundColor(Color::Black),
            SetForegroundColor(Color::White),
        )
        .expect("Problem resetting colors");
    }

    fn update(&self, game: &Game, change: &SnakeChange) {
        for Coordinate { x, y } in game.snake.iter().skip(1).take(1) {
            Console::snake_tail(Color::Yellow, Color::Green, x, y);
        }

        if let Some(Coordinate { x, y }) = change.added {
            Console::snake_head(Color::Green, Color::DarkGreen, &x, &y);
        }

        if let Some(Coordinate { x, y }) = change.removed {
            execute!(
                stdout(),
                MoveTo((x + 1).try_into().unwrap(), (y + 1).try_into().unwrap()),
                SetBackgroundColor(Color::Black),
                Print(" "),
            )
            .expect("Problem clearing snake");
        }

        Console::score(
            game.snake.len(),
            Color::White,
            Color::Black,
            game.width.into(),
            game.height.into(),
        );
    }
}

impl GameInput for Console {
    fn poll(&self) -> Option<Turn> {
        if poll(Duration::from_secs(0)).unwrap() {
            return match read().unwrap() {
                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }) => Some(Turn::Left),
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                }) => Some(Turn::Right),
                _ => None,
            };
        }
        None
    }
}

fn main() {
    let console = Console {};
    let mut game = Game::new(&30, &10);
    console.initialize(&game);
    let mut counter = 0u32;
    while !game.game_over {
        sleep(Duration::from_millis(200));
        game_step(&mut counter, &mut game, &console, &console);
    }
}
