use ggez::event::EventHandler;
use ggez::{Context, ContextBuilder, event, GameError, GameResult, graphics};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::glam::Vec2;
use ggez::graphics::{Canvas, Color, Drawable, DrawParam};
use ggez::input::keyboard::{KeyCode, KeyInput};
use once_cell::sync::Lazy;
use rand::rngs::ThreadRng;
use rand::{Rng, thread_rng};
use crate::snake_game::{Ate, Direction, Food, Position, Snake};

pub const GRID_SIZE: (i16, i16) = (10, 10);
pub const GRID_CELL_SIZE: (i16, i16) = (24, 24);

pub static MAX_DISTANCE: Lazy<f64> = Lazy::new(|| ((GRID_SIZE.0.pow(2) + GRID_SIZE.1.pow(2)) as f64).sqrt());

pub const SCREEN_SIZE: (f32, f32) = (
    (GRID_SIZE.0 * GRID_CELL_SIZE.0) as f32,
    (GRID_SIZE.1 * GRID_CELL_SIZE.1) as f32
);

pub const FPS: u32 = 8;
struct SnakeGameState {
    snake: Snake,
    food: Food,
    game_over: bool,
    rng: ThreadRng
}

impl SnakeGameState {
    pub fn new() -> Self {
        let snake_pos: Position = (GRID_SIZE.0 / 4, GRID_SIZE.1 / 2).into();

        let rng = thread_rng();

        let mut game_state = SnakeGameState {
            snake: Snake::new(snake_pos),
            food: Food::new(Position::new(0, 0)),
            game_over: false,
            rng
        };

        game_state.food = game_state.generate_new_food();

        game_state
    }

    pub fn generate_new_food(&mut self) -> Food {
        let mut new_food_position: Position = (
            self.rng.gen_range(0..GRID_SIZE.0),
            self.rng.gen_range(0..GRID_SIZE.1)
        ).into();

        while self.snake.is_in_position(new_food_position) {
            new_food_position = (
            self.rng.gen_range(0..GRID_SIZE.0),
            self.rng.gen_range(0..GRID_SIZE.1)
            ).into();
        }

        Food::new(new_food_position)
    }
}

impl EventHandler<GameError> for SnakeGameState {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        while ctx.time.check_update_time(FPS) {
            if !self.game_over {
                self.snake.update_state(&self.food);

                if let Some(ate) = self.snake.get_ate() {
                    match ate {
                        Ate::Food => self.food = self.generate_new_food(),
                        Ate::Itself | Ate::Border => self.game_over = true
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(ctx, Color::from_rgb(255, 255, 255));

        if self.game_over {
            let mut text = graphics::Text::new("Game Over!");
            text.set_scale(48.);

            let (text_width, text_height) = match text.dimensions(ctx) {
                Some(rectangle ) => (rectangle.w, rectangle.h),
                None => return Err(GameError::CustomError("Could not retrieve text's bounding rectangle".into()))
            };
            canvas.draw(
                &text,
                DrawParam::new()
                    .dest(Vec2::new((SCREEN_SIZE.0  - text_width) / 2.0,
                                    (SCREEN_SIZE.1 - text_height) / 2.0))
                    .color(Color::from_rgb(0, 0, 0))
            )
        } else {
            self.snake.draw(&mut canvas);
            self.food.draw(&mut canvas);
        }

        canvas.finish(ctx)?;

        ggez::timer::yield_now();

        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeated: bool) -> Result<(), GameError> {
        if let Some(dir) = input.keycode.and_then(Direction::from_key) {
            self.snake.move_in_dir(dir);
        }

        if self.game_over {
            if input.keycode == Some(KeyCode::Escape) {
            ctx.request_quit();
            }
        }

        Ok(())
    }
}

pub fn play_game() -> GameResult {
    let (ctx, events_loop) = ContextBuilder::new("Snake game", "Siemano")
        .window_setup(WindowSetup::default().title("Snake game"))
        .window_mode(WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;

    let state = SnakeGameState::new();

    event::run(ctx, events_loop, state)
}

