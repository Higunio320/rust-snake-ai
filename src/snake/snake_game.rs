use std::collections::VecDeque;
use std::f64::consts::FRAC_PI_4;
use ggez::graphics::{Canvas, Color, DrawParam, Quad, Rect};
use ggez::input::keyboard::{KeyCode};
use once_cell::sync::Lazy;
use crate::ai::snake_trainer::Move;
use crate::visualisation::game_constants::{GRID_CELL_SIZE,GRID_SIZE};

static SIN_45: Lazy<f64> = Lazy::new(|| FRAC_PI_4.sin());
static COS_45: Lazy<f64> = Lazy::new(|| FRAC_PI_4.cos());

#[derive(Copy, PartialEq, Clone, Debug)]
pub struct Position {
    pub(crate) x: i16,
    pub(crate) y: i16
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

    pub fn get_distance_from_pos(&self, position: &Position) -> f64 {
        (((self.x - position.x).pow(2) + (self.y - position.y).pow(2)) as f64).sqrt()
    }

    pub fn get_distance(&self) -> f64 {
        ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt()
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
    position: Position,
    direction: Direction
}

impl Segment {
    pub fn new(position: Position, direction: Direction) -> Self {
        Segment {position, direction}
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

    pub fn get_position(&self) -> Position {
        self.position
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

#[derive(PartialEq, Debug)]
pub struct Distances {
    pub(crate) top: DistanceInfo,
    pub(crate) right: DistanceInfo,
    pub(crate) bottom: DistanceInfo,
    pub(crate) left: DistanceInfo,
    pub(crate) top_right: DistanceInfo,
    pub(crate) bottom_right: DistanceInfo,
    pub(crate) bottom_left: DistanceInfo,
    pub(crate) top_left: DistanceInfo
}

#[derive(Debug)]
pub struct DistanceInfo {
    pub(crate) distance_to_wall: f64,
    pub(crate) distance_to_apple: f64,
    pub(crate) distance_to_body: f64
}

impl PartialEq for DistanceInfo {
    fn eq(&self, other: &Self) -> bool {
        Self::equals_with_error(self.distance_to_wall, other.distance_to_wall, 0.0000001) &&
            Self::equals_with_error(self.distance_to_body, other.distance_to_body, 0.0000001) &&
            Self::equals_with_error(self.distance_to_apple, other.distance_to_apple, 0.0000001)
    }
}

impl DistanceInfo {
    fn equals_with_error(a: f64, b: f64, error: f64) -> bool {
        a > b - error && a < b + error
    }
}

impl From<(f64, f64, f64)> for DistanceInfo {
    fn from(value: (f64, f64, f64)) -> Self {
        DistanceInfo {
            distance_to_wall: value.0,
            distance_to_apple: value.1,
            distance_to_body: value.2
        }
    }
}

impl Snake {
    pub fn new(position: Position) -> Self {
        let mut body = VecDeque::new();

        body.push_back(Segment::new((position.x - 1, position.y).into(), Direction::RIGHT));
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

        self.body.push_front(Segment::new(self.head.position, self.head.direction));

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

    pub fn move_in_dir_with_move(&mut self, move_dir: Move) {
        let direction = match move_dir {
            Move::FORWARD => self.head.direction,
            Move::LEFT => match self.head.direction {
                Direction::UP => Direction::LEFT,
                Direction::LEFT => Direction::DOWN,
                Direction::DOWN => Direction::RIGHT,
                Direction::RIGHT => Direction::UP
            },
            Move::RIGHT => match self.head.direction {
                Direction::UP => Direction::RIGHT,
                Direction::RIGHT => Direction::DOWN,
                Direction::DOWN => Direction::LEFT,
                Direction::LEFT => Direction::UP
            }
        };

        self.move_in_dir(direction);
    }

    pub fn get_distances(&self, food: &Food) -> Distances {
        let top_distance = self.head.position.y as f64;
        let top_body = self.body.iter()
            .filter(|segment| segment.position.x == self.head.position.x && segment.position.y < self.head.position.y)
            .next().map(|_| 1.0).unwrap_or_else(|| 0.0);

        let top_apple = if food.position.x == self.head.position.x &&
            food.position.y < self.head.position.y {
            1.0
        } else {
            0.0
        };

        let bottom_distance = (GRID_SIZE.1 - self.head.position.y - 1) as f64;
        let bottom_body = self.body.iter()
            .filter(|segment| segment.position.x == self.head.position.x && segment.position.y > self.head.position.y)
            .next().map(|_| 1.0).unwrap_or_else(|| 0.0);

        let bottom_apple = if food.position.x == self.head.position.x &&
            food.position.y > self.head.position.y {
            1.0
        } else {
            0.0
        };

        let right_distance = (GRID_SIZE.0 - self.head.position.x - 1) as f64;
        let right_body = self.body.iter()
            .filter(|segment| segment.position.x > self.head.position.x && segment.position.y == self.head.position.y)
            .next().map(|_| 1.0).unwrap_or_else(|| 0.0);

        let right_apple = if food.position.x > self.head.position.x &&
            food.position.y == self.head.position.y {
            1.0
        } else {
            0.0
        };

        let left_distance = self.head.position.x as f64;
        let left_body = self.body.iter()
            .filter(|segment| segment.position.x < self.head.position.x && segment.position.y == self.head.position.y)
            .next().map(|_| 1.0).unwrap_or_else(|| 0.0);

        let left_apple = if food.position.x < self.head.position.x &&
            food.position.y == self.head.position.y {
            1.0
        } else {
            0.0
        };

        let top: DistanceInfo = (top_distance, top_apple, top_body).into();
        let bottom: DistanceInfo = (bottom_distance, bottom_apple, bottom_body).into();
        let right: DistanceInfo = (right_distance, right_apple, right_body).into();
        let left: DistanceInfo = (left_distance, left_apple, left_body).into();

        let top_right = self.get_distance_in_direction(&food.position, top_distance, right_distance, *SIN_45, *COS_45);
        let bottom_right = self.get_distance_in_direction(&food.position, bottom_distance, right_distance, -*SIN_45, *COS_45);
        let bottom_left = self.get_distance_in_direction(&food.position, bottom_distance, left_distance, -*SIN_45, -*COS_45);
        let top_left = self.get_distance_in_direction(&food.position, top_distance, left_distance, *SIN_45, -*COS_45);

        Distances {
            top,
            bottom,
            right,
            left,
            top_right,
            bottom_right,
            bottom_left,
            top_left
        }
    }

    pub fn get_head_coordinates(&self) -> Position {
        self.head.position
    }

    fn get_distance_in_direction(&self, food_pos: &Position, top_bottom_dist: f64, left_right_dist: f64, vec_sin: f64, vec_cos: f64) -> DistanceInfo {
        let distance;
        if top_bottom_dist < left_right_dist {
            distance = (top_bottom_dist / vec_sin).abs();
        } else {
            distance = (left_right_dist / vec_cos).abs();
        }

        let apple_vec = Position::new(food_pos.x - self.head.position.x,  self.head.position.y - food_pos.y);

        let apple_vec_len = apple_vec.get_distance();

        let apple = if equal_with_error(apple_vec.x as f64 / apple_vec_len, vec_cos, 0.00001) &&
            equal_with_error(apple_vec.y as f64 / apple_vec_len, vec_sin, 0.00001) {
            1.0
        } else {
            0.0
        };

        let body = self.body.iter()
            .filter(|segment| {
                let distance = segment.position.get_distance_from_pos(&self.head.position);
                equal_with_error((segment.position.x - self.head.position.x) as f64 / distance, vec_cos, 0.00001) &&
                    equal_with_error((self.head.position.y - segment.position.y) as f64 / distance, vec_sin, 0.00001)
            })
            .next().map(|_| 1.0).unwrap_or_else(|| 0.0);

        DistanceInfo {
            distance_to_wall: distance,
            distance_to_apple: apple,
            distance_to_body: body
        }
    }

    pub fn get_current_direction(&self) -> Direction {
        self.head.direction
    }

    pub fn get_tail_direction(&self) -> Direction {
        self.body[self.body.len() - 1].direction
    }
}

fn equal_with_error(first_value: f64, second_value: f64, error: f64) -> bool {
    return second_value >= first_value - error && second_value <= first_value + error
}
