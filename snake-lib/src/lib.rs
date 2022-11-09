use std::collections::{HashSet, VecDeque};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone)]
pub enum Turn {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

type Snake = VecDeque<Coordinate>;

pub struct Game {
    pub snake: Snake,
    active_cells: HashSet<Coordinate>,
    direction: Direction,
    pub width: u16,
    pub height: u16,
    growth: u32,
    pub game_over: bool,
}

#[derive(Debug)]
pub struct SnakeChange {
    pub removed: Option<Coordinate>,
    pub added: Option<Coordinate>,
}

impl Coordinate {
    fn out_of_bounds(&self, game: &Game) -> bool {
        self.x < 0 || self.x >= game.width.into() || self.y < 0 || self.y >= game.height.into()
    }
}

impl Direction {
    fn turn(&self, turn: &Turn) -> Direction {
        match (self, turn) {
            (Direction::East, Turn::Left) => Direction::North,
            (Direction::East, Turn::Right) => Direction::South,
            (Direction::West, Turn::Left) => Direction::South,
            (Direction::West, Turn::Right) => Direction::North,
            (Direction::North, Turn::Left) => Direction::West,
            (Direction::North, Turn::Right) => Direction::East,
            (Direction::South, Turn::Left) => Direction::East,
            (Direction::South, Turn::Right) => Direction::West,
        }
    }
}

impl Coordinate {
    fn advance(&self, direction: &Direction) -> Coordinate {
        match direction {
            Direction::North => Coordinate {
                x: self.x,
                y: self.y - 1,
            },
            Direction::South => Coordinate {
                x: self.x,
                y: self.y + 1,
            },
            Direction::East => Coordinate {
                x: self.x + 1,
                y: self.y,
            },
            Direction::West => Coordinate {
                x: self.x - 1,
                y: self.y,
            },
        }
    }
}

impl Game {
    pub fn new(width: &u16, height: &u16) -> Game {
        Game {
            snake: VecDeque::from([Coordinate {
                x: (width / 2).into(),
                y: (height / 2).into(),
            }]),
            active_cells: HashSet::from([Coordinate {
                x: (width / 2).into(),
                y: (height / 2).into(),
            }]),
            direction: Direction::East,
            width: *width,
            height: *height,
            growth: 3,
            game_over: false,
        }
    }

    pub fn turn(&mut self, turn: &Turn) {
        self.direction = self.direction.turn(turn);
    }

    pub fn grow(&mut self, n: &u32) {
        self.growth += n;
    }

    pub fn advance(&mut self) -> SnakeChange {
        let mut removed = None;
        let mut added = None;
        if !self.game_over {
            let new_front = self.snake.front().unwrap().advance(&self.direction);

            if self.growth > 0 {
                self.growth -= 1;
            } else {
                let r = self.snake.pop_back().unwrap();
                self.active_cells.remove(&r);
                removed = Some(r);
            }

            if new_front.out_of_bounds(self) || self.active_cells.contains(&new_front) {
                self.game_over = true;
            } else {
                self.active_cells.insert(new_front.clone());
                self.snake.push_front(new_front.clone());
                added = Some(new_front);
            }
        }
        SnakeChange { removed, added }
    }
}

pub trait GameDisplay {
    fn initialize(&self, game: &Game);
    fn game_over(&self, game: &Game);
    fn update(&self, game: &Game, change: &SnakeChange);
}
pub trait GameInput {
    fn poll(&self) -> Option<Turn>;
}

pub fn game_step<D: GameDisplay, I: GameInput>(
    counter: &mut u32,
    game: &mut Game,
    display: &D,
    input: &I,
) {
    if let Some(t) = input.poll() {
        game.turn(&t);
    }

    if *counter > 20u32 {
        *counter = 0u32;
        game.grow(&3);
    } else {
        *counter += 1;
    }

    let change = game.advance();
    display.update(game, &change);
    if game.game_over {
        display.game_over(game);
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::{
        game_step, Coordinate, Direction, Game, GameDisplay, GameInput, SnakeChange, Turn,
    };
    use test_case::test_case;
    #[test_case(Direction::North)]
    #[test_case(Direction::South)]
    #[test_case(Direction::East)]
    #[test_case(Direction::West)]
    fn turn_changes_direction(d: Direction) {
        assert_ne!(d.turn(&Turn::Left), d);
    }

    #[test_case(Direction::North)]
    #[test_case(Direction::South)]
    #[test_case(Direction::East)]
    #[test_case(Direction::West)]
    fn turn_left_then_right(d: Direction) {
        assert_eq!(d.turn(&Turn::Left).turn(&Turn::Right), d);
    }

    #[test_case(Direction::North)]
    #[test_case(Direction::South)]
    #[test_case(Direction::East)]
    #[test_case(Direction::West)]
    fn turn_left_four_times(d: Direction) {
        assert_eq!(
            d.turn(&Turn::Left)
                .turn(&Turn::Left)
                .turn(&Turn::Left)
                .turn(&Turn::Left),
            d
        );
    }

    #[test_case(Direction::North)]
    #[test_case(Direction::South)]
    #[test_case(Direction::East)]
    #[test_case(Direction::West)]
    fn advance_changes_coordinate_by_one(d: Direction) {
        let c = Coordinate { x: 100, y: 200 };
        let advanced = c.advance(&d);
        assert_ne!(advanced, c);
        assert_eq!((c.x - advanced.x).abs() + (c.y - advanced.y).abs(), 1);
    }

    #[test]
    fn advance_north_east_south_west() {
        assert_eq!(
            Coordinate { x: 100, y: 200 }
                .advance(&Direction::North)
                .advance(&Direction::East)
                .advance(&Direction::South)
                .advance(&Direction::West),
            Coordinate { x: 100, y: 200 }
        )
    }

    #[test_case(100, 100, 50, 50)]
    #[test_case(99, 99, 49, 49)]
    #[test_case(100, 200, 50, 100)]
    fn snake_starts_in_centre_of_board(width: u16, height: u16, x: i32, y: i32) {
        let game = Game::new(&width, &height);
        assert_eq!(game.snake.len(), 1);
        assert_eq!(game.snake.front().unwrap(), &Coordinate { x, y });
    }

    #[test]
    fn snake_initially_grows_to_length_four() {
        let mut game = Game::new(&100, &100);
        assert_eq!(game.snake.len(), 1);
        game.advance();
        assert_eq!(game.snake.len(), 2);
        game.advance();
        assert_eq!(game.snake.len(), 3);
        game.advance();
        assert_eq!(game.snake.len(), 4);
        for _ in 0..5 {
            game.advance();
            assert_eq!(game.snake.len(), 4);
        }
    }

    #[test]
    fn snake_can_grow_to_any_length() {
        let mut game = Game::new(&100, &100);
        game.grow(&6);
        for i in 1..=10 {
            assert_eq!(game.snake.len(), i);
            game.advance();
        }

        for _ in 0..5 {
            game.advance();
            assert_eq!(game.snake.len(), 10);
        }
    }

    #[test]
    fn snake_hits_wall_game_over() {
        let mut game = Game::new(&20, &20);
        assert!(!game.game_over);
        for _ in 0..9 {
            game.advance();
            assert!(!game.game_over);
        }
        game.advance();
        assert!(game.game_over);
    }

    #[test]
    fn snake_bites_self_game_over() {
        let mut game = Game::new(&20, &20);
        game.grow(&10);
        assert!(!game.game_over);
        for _ in 0..3 {
            game.advance();
            assert!(!game.game_over);
            game.turn(&Turn::Left);
        }
        game.turn(&Turn::Left);
        game.advance();
        assert!(game.game_over);
    }

    #[test]
    fn advance_returns_changed_coordinates() {
        let mut game = Game::new(&20, &20);

        let SnakeChange { added, removed } = game.advance();
        assert_eq!(added, Some(Coordinate { x: 11, y: 10 }));
        assert!(removed.is_none());

        let SnakeChange { added, removed } = game.advance();
        assert_eq!(added, Some(Coordinate { x: 12, y: 10 }));
        assert!(removed.is_none());

        let SnakeChange { added, removed } = game.advance();
        assert_eq!(added, Some(Coordinate { x: 13, y: 10 }));
        assert!(removed.is_none());

        let SnakeChange { added, removed } = game.advance();
        assert_eq!(added, Some(Coordinate { x: 14, y: 10 }));
        assert_eq!(removed, Some(Coordinate { x: 10, y: 10 }));

        game.turn(&Turn::Right);
        let SnakeChange { added, removed } = game.advance();
        assert_eq!(added, Some(Coordinate { x: 14, y: 11 }));
        assert_eq!(removed, Some(Coordinate { x: 11, y: 10 }));

        let SnakeChange { added, removed } = game.advance();
        assert_eq!(added, Some(Coordinate { x: 14, y: 12 }));
        assert_eq!(removed, Some(Coordinate { x: 12, y: 10 }));
    }

    struct MockDisplay {
        pub calls: RefCell<Vec<String>>,
    }

    impl MockDisplay {
        fn new() -> MockDisplay {
            MockDisplay {
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl GameDisplay for MockDisplay {
        fn game_over(&self, _game: &Game) {
            self.calls.borrow_mut().push("game_over".to_string());
        }
        fn initialize(&self, _game: &Game) {
            self.calls.borrow_mut().push("initialize".to_string());
        }
        fn update(&self, _game: &Game, change: &SnakeChange) {
            self.calls
                .borrow_mut()
                .push(format!("update({:?})", change));
        }
    }

    struct MockInput {
        imp: RefCell<MockInputImpl>,
    }

    struct MockInputImpl {
        inputs: Vec<Option<Turn>>,
        i: usize,
    }

    impl MockInput {
        fn new(inputs: Vec<Option<Turn>>) -> MockInput {
            MockInput {
                imp: RefCell::new(MockInputImpl { inputs, i: 0 }),
            }
        }
        fn no_input() -> MockInput {
            MockInput {
                imp: RefCell::new(MockInputImpl {
                    inputs: Vec::new(),
                    i: 0,
                }),
            }
        }
    }

    impl MockInputImpl {
        fn poll(&mut self) -> Option<Turn> {
            if self.i < self.inputs.len() {
                let r = self.inputs.get(self.i).unwrap();
                self.i += 1;
                return r.clone();
            } else {
                return None;
            }
        }
    }

    impl GameInput for MockInput {
        fn poll(&self) -> Option<Turn> {
            self.imp.borrow_mut().poll()
        }
    }

    #[test]
    fn counter_increases_each_step() {
        let mut game = Game::new(&20, &20);
        let mut counter = 0;
        let mut prev_counter = 0;
        for _ in 0..10 {
            game_step(
                &mut counter,
                &mut game,
                &MockDisplay::new(),
                &MockInput::no_input(),
            );
            assert_eq!(counter, prev_counter + 1);
            prev_counter = counter;
        }
    }
    #[test]
    fn snake_grows_by_three_when_counter_is_more_than_twenty() {
        let mut game = Game::new(&20, &20);
        assert_eq!(game.growth, 3);
        let mut counter = 21;
        game_step(
            &mut counter,
            &mut game,
            &MockDisplay::new(),
            &MockInput::no_input(),
        );
        assert_eq!(game.growth, 5);
        assert_eq!(counter, 0);
    }

    #[test]
    fn display_receives_updates() {
        let mut game = Game::new(&20, &20);
        let mut counter = 0;
        let display = MockDisplay::new();
        for _ in 0..5 {
            game_step(&mut counter, &mut game, &display, &MockInput::no_input());
        }
        assert_eq!(
            *display.calls.borrow(),
            vec![
                "update(SnakeChange { removed: None, added: Some(Coordinate { x: 11, y: 10 }) })",
                "update(SnakeChange { removed: None, added: Some(Coordinate { x: 12, y: 10 }) })",
                "update(SnakeChange { removed: None, added: Some(Coordinate { x: 13, y: 10 }) })",
                "update(SnakeChange { removed: Some(Coordinate { x: 10, y: 10 }), added: Some(Coordinate { x: 14, y: 10 }) })",
                "update(SnakeChange { removed: Some(Coordinate { x: 11, y: 10 }), added: Some(Coordinate { x: 15, y: 10 }) })"
            ]
        );
    }

    #[test]
    fn display_receives_game_over() {
        let mut game = Game::new(&20, &20);
        game.grow(&10);
        let mut counter = 0;
        let display = MockDisplay::new();
        while !game.game_over {
            game_step(
                &mut counter,
                &mut game,
                &display,
                &MockInput::new(vec![Some(Turn::Left)]),
            );
        }
        assert_eq!(
            *display.calls.borrow(),
            vec![
                "update(SnakeChange { removed: None, added: Some(Coordinate { x: 10, y: 9 }) })",
                "update(SnakeChange { removed: None, added: Some(Coordinate { x: 9, y: 9 }) })",
                "update(SnakeChange { removed: None, added: Some(Coordinate { x: 9, y: 10 }) })",
                "update(SnakeChange { removed: None, added: None })",
                "game_over"
            ]
        );
    }

    #[test]
    fn input_steers_snake() {
        let mut game = Game::new(&20, &20);
        let mut counter = 0;
        let display = MockDisplay::new();
        let input = MockInput::new(vec![
            None,
            None,
            Some(Turn::Left),
            None,
            Some(Turn::Right),
            None,
        ]);
        for _ in 0..7 {
            game_step(&mut counter, &mut game, &display, &input);
        }
        assert_eq!(
            *display.calls.borrow(),
            vec![
                "update(SnakeChange { removed: None, added: Some(Coordinate { x: 11, y: 10 }) })",
                "update(SnakeChange { removed: None, added: Some(Coordinate { x: 12, y: 10 }) })",
                "update(SnakeChange { removed: None, added: Some(Coordinate { x: 12, y: 9 }) })",
                "update(SnakeChange { removed: Some(Coordinate { x: 10, y: 10 }), added: Some(Coordinate { x: 12, y: 8 }) })",
                "update(SnakeChange { removed: Some(Coordinate { x: 11, y: 10 }), added: Some(Coordinate { x: 13, y: 8 }) })",
                "update(SnakeChange { removed: Some(Coordinate { x: 12, y: 10 }), added: Some(Coordinate { x: 14, y: 8 }) })",
                "update(SnakeChange { removed: Some(Coordinate { x: 12, y: 9 }), added: Some(Coordinate { x: 15, y: 8 }) })"
            ]
        );
    }
}
