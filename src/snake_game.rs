use std::collections::VecDeque;
use ggez::graphics::{Canvas, Color, DrawParam, Quad, Rect};
use ggez::input::keyboard::{KeyCode};
use crate::game::{GRID_CELL_SIZE, GRID_SIZE};

#[derive(Copy, PartialEq, Clone)]
pub struct Position {
    x: i16,
    y: i16
}

impl Position {
    pub fn new(x: i16, y: i16) -> Self {
        Position {x, y}
    }

    pub fn make_a_move(&mut self, direction: Direction) {
        match direction {
            Direction::UP => self.y -= 1,
            Direction::DOWN => self.y += 1,
            Direction::LEFT => self.x -= 1,
            Direction::RIGHT => self.x += 1
        }
    }


}

impl From<Position> for Rect {
    fn from(value: Position) -> Self {
        Rect::new_i32(
            (value.x * GRID_CELL_SIZE.0) as i32,
            (value.y * GRID_CELL_SIZE.1) as i32,
            GRID_CELL_SIZE.0 as i32,
            GRID_CELL_SIZE.1 as i32)
    }
}

impl From<(i16,i16)> for Position {
    fn from(value: (i16, i16)) -> Self {
        Position::new(value.0, value.1)
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum Direction {
    UP,
    LEFT,
    RIGHT,
    DOWN
}

impl Direction {
    pub fn inverse(&self) -> Self {
        match self {
            Direction::UP => Direction::DOWN,
            Direction::DOWN => Direction::UP,
            Direction::LEFT => Direction::RIGHT,
            Direction::RIGHT => Direction::LEFT
        }
    }

    pub fn from_key(code: KeyCode) -> Option<Self> {
        match code {
            KeyCode::W | KeyCode::Up => Some(Direction::UP),
            KeyCode::A | KeyCode::Left => Some(Direction::LEFT),
            KeyCode::S | KeyCode::Down => Some(Direction::DOWN),
            KeyCode::D | KeyCode::Right => Some(Direction::RIGHT),
            _ => None
        }
    }
}

struct Head {
    position: Position,
    direction: Direction
}

impl Head {
    pub fn new(position: Position, direction: Direction) -> Self {
        Head {
            position,
            direction
        }
    }

    pub fn draw(&self, canvas: &mut Canvas) {
        let color = Color::from_rgb(15, 74, 4);

        canvas.draw(
            &Quad,
            DrawParam::new()
                .dest_rect(self.position.into())
                .color(color)
        );
    }
}

struct Segment {
    position: Position
}

impl Segment {
    pub fn new(position: Position) -> Self {
        Segment {position}
    }

    pub fn draw(&self, canvas: &mut Canvas) {
        let color = Color::from_rgb(6, 140, 8);

        canvas.draw(
            &Quad,
            DrawParam::new()
                .dest_rect(self.position.into())
                .color(color)
        );
    }
}

pub struct Food {
    position: Position
}

impl Food {
    pub fn new(position: Position) -> Self {
        Food {position}
    }

    pub fn draw(&self, canvas: &mut Canvas) {
        let color = Color::from_rgb(255, 0, 0);

        canvas.draw(
            &Quad,
            DrawParam::new()
                .dest_rect(self.position.into())
                .color(color)
        );
    }
}

#[derive(Copy, Clone)]
pub enum Ate {
    Itself,
    Food,
    Border
}

pub struct Snake {
    head: Head,
    body: VecDeque<Segment>,
    ate: Option<Ate>,
    last_dir: Direction,
    next_dir: Option<Direction>
}

impl Snake {
    pub fn new(position: Position) -> Self {
        let mut body = VecDeque::new();

        body.push_back(Segment::new((position.x - 1, position.y).into()));
        Snake {
            head: Head::new(position, Direction::RIGHT),
            last_dir: Direction::RIGHT,
            body,
            ate: None,
            next_dir: None
        }
    }

    pub fn eats(&self, food: &Food) -> bool {
        self.head.position == food.position
    }

    pub fn eats_self(&self) -> bool {
        for segment in &self.body {
            if self.head.position == segment.position {
                return true
            }
        }
        false
    }

    pub fn eats_border(&self) -> bool {
        match self.head.direction {
            Direction::LEFT => self.head.position.x < 0,
            Direction::UP => self.head.position.y < 0,
            Direction::RIGHT => self.head.position.x >= GRID_SIZE.0,
            Direction::DOWN => self.head.position.y >= GRID_SIZE.1
        }
    }

    pub fn is_in_position(&self, position: Position) -> bool {
        if self.head.position == position {
            return true
        }

        for segment in &self.body {
            if segment.position == position {
                return true
            }
        }

        false
    }

    pub fn update_state(&mut self, food: &Food) {
        if self.last_dir == self.head.direction && self.next_dir.is_some() {
            self.head.direction = self.next_dir.unwrap();
            self.next_dir = None
        }

        self.body.push_front(Segment::new(self.head.position));

        self.head.position.make_a_move(self.head.direction);

        if self.eats(food) {
            self.ate = Some(Ate::Food)
        } else if self.eats_border() {
            self.ate = Some(Ate::Border)
        } else if self.eats_self() {
            self.ate = Some(Ate::Itself)
        } else {
            self.ate = None
        }

        match self.ate {
            Some(ate) => {
                match ate {
                    Ate::Food => {},
                    _ => {self.body.pop_back();}
                }
            },
            None => {
                self.body.pop_back();
            }
        }

        self.last_dir = self.head.direction;
    }

    pub fn draw(&self, canvas: &mut Canvas) {
        for segment in &self.body {
            segment.draw(canvas)
        }

        self.head.draw(canvas);
    }

    pub fn get_ate(&self) -> Option<Ate> {
        self.ate
    }

    pub fn move_in_dir(&mut self, new_direction: Direction) {
        if self.head.direction != self.last_dir && new_direction.inverse() != self.head.direction {
            self.next_dir = Some(new_direction)
        } else if new_direction.inverse() != self.last_dir {
            self.head.direction = new_direction
        }
    }
}

