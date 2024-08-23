use std::fmt::format;
use ggez::event::EventHandler;
use ggez::{Context, ContextBuilder, event, GameError, GameResult, graphics};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::glam::Vec2;
use ggez::graphics::{Canvas, Color, DrawParam, FontData, Mesh};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::mint::Point2;
use rand::prelude::ThreadRng;
use rand::thread_rng;
use crate::game::{FPS, GAME_SCREEN_SIZE, GRID_SIZE, SCREEN_SIZE};
use crate::neural_network::{NeuralNetwork, NeuralNetworkOptions};
use crate::snake_game::{Ate, DistanceInfo, Distances, Food, Snake};
use crate::snake_trainer::{generate_network_input, generate_new_food, generate_random_position, interpret_network_output};

struct MLSnakeGameState {
    current_game_index: usize,
    weights: Vec<Vec<f64>>,
    snake: Snake,
    food: Food,
    game_over: bool,
    rng: ThreadRng,
    neural_network: NeuralNetwork,
    current_score: u16,
    stop: bool,
    distances: Distances
}

impl MLSnakeGameState {
    fn new(neural_network_options: NeuralNetworkOptions, weights: Vec<Vec<f64>>) -> Self {
        let snake_pos = generate_random_position();

        let rng = thread_rng();

        let current_game_index = (0.95 * weights.len() as f64) as usize;

        let neural_network = NeuralNetwork::new_with_weights(weights[0].clone(), neural_network_options).unwrap();

        let snake = Snake::new(snake_pos);

        let food = generate_new_food(&snake);

        let current_score = 0_u16;

        let distances = snake.get_distances(&food);

        MLSnakeGameState {
            snake,
            food,
            neural_network,
            rng,
            game_over: false,
            current_game_index,
            weights,
            current_score,
            stop: false,
            distances: distances
        }
    }
}

impl EventHandler<GameError> for MLSnakeGameState {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        while ctx.time.check_update_time(FPS) {
            if !self.game_over && !self.stop {

                let input = generate_network_input(&self.snake, &self.food);

                let output = self.neural_network.get_output(input).unwrap();

                let move_dir = interpret_network_output(&output);

                self.snake.move_in_dir(move_dir);

                self.snake.update_state(&self.food);

                self.distances = self.snake.get_distances(&self.food);

                if let Some(ate) = self.snake.get_ate() {
                    match ate {
                        Ate::Food => {
                            self.food = generate_new_food(&self.snake);
                            self.current_score += 1;
                        },
                        Ate::Itself | Ate::Border => self.game_over = true
                    }
                }
            } else {
                if self.current_game_index < self.weights.len() {

                    let snake_pos = generate_random_position();

                    let snake = Snake::new(snake_pos);

                    let food = generate_new_food(&snake);

                    self.neural_network.update_weights(self.weights[self.current_game_index].clone());

                    self.current_game_index += 1;

                    self.snake = snake;

                    self.food = food;

                    self.current_score = 0;

                    self.game_over = false;
                    self.stop = false;
                } else {
                    ctx.request_quit();
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(ctx, Color::from_rgb(255, 255, 255));

        self.snake.draw(&mut canvas);
        self.food.draw(&mut canvas);

        self.draw_distances(&mut canvas)?;

        let mut text = graphics::Text::new(format!("Current gen: {}, current score: {}",
                                                   self.current_game_index + 1, self.current_score));

        text.set_scale(28.);

        canvas.draw(
            &text,
            DrawParam::new()
                .dest(Vec2::new(GAME_SCREEN_SIZE.0 + 20.0, 5.0))
                .color(Color::from_rgb(0, 0, 0))
        );

        self.draw_border(ctx, &mut canvas)?;

        canvas.finish(ctx)?;

        ggez::timer::yield_now();

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeated: bool) -> Result<(), GameError> {
        if let Some(code) = input.keycode {
            match code {
                KeyCode::Right => self.stop = true,
                _ => {}
            }
        };

        Ok(())
    }
}

impl MLSnakeGameState {
    fn draw_border(&self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        let thickness = 2.0;
        let color = Color::from_rgb(0, 0, 0);
        let top_line = Mesh::new_line(
            ctx,
            &[Point2::from_slice(&[0.0, 0.0]), [GAME_SCREEN_SIZE.0, 0.0].into()],
            thickness,
            color
        )?;

        let bottom_line = Mesh::new_line(
            ctx,
            &[Point2::from_slice(&[0.0, GAME_SCREEN_SIZE.1]), [GAME_SCREEN_SIZE.0, GAME_SCREEN_SIZE.1].into()],
            thickness,
            color
        )?;

        let left_line = Mesh::new_line(
            ctx,
            &[Point2::from_slice(&[0.0, 0.0]), [0.0, GAME_SCREEN_SIZE.1].into()],
            thickness,
            color
        )?;

        let right_line = Mesh::new_line(
            ctx,
            &[Point2::from_slice(&[GAME_SCREEN_SIZE.0, 0.0]), [GAME_SCREEN_SIZE.0, GAME_SCREEN_SIZE.1].into()],
            thickness,
            color
        )?;

        canvas.draw(&top_line, DrawParam::default());
        canvas.draw(&left_line, DrawParam::default());
        canvas.draw(&bottom_line, DrawParam::default());
        canvas.draw(&right_line, DrawParam::default());

        Ok(())
    }

    fn draw_distances(&self, canvas: &mut Canvas) -> Result<(), GameError> {
        let x = GAME_SCREEN_SIZE.0 + 50.0;

        let mut y = 80.0;
        let head_coordinates = self.snake.get_head_coordinates();
        let mut text = graphics::Text::new(format!("Head pos: {} {}", head_coordinates.x, head_coordinates.y));
        text.set_scale(15.0);

        canvas.draw(
            &text,
            DrawParam::new()
                .dest(Vec2::new(x, y))
                .color(Color::from_rgb(0, 0, 0))
        );

        y += 15.0 + 5.0;

        let food_coordinates = self.food.get_position();
        let mut text = graphics::Text::new(format!("Food pos: {} {}", food_coordinates.x, food_coordinates.y));
        text.set_scale(15.0);

        canvas.draw(
            &text,
            DrawParam::new()
                .dest(Vec2::new(x, y))
                .color(Color::from_rgb(0, 0, 0))
        );

        y += 15.0 + 5.0;

        self.draw_distance_info(canvas, &mut y, 5.0, &self.distances.top, "Top", x)?;
        self.draw_distance_info(canvas, &mut y, 5.0, &self.distances.top_right, "Top right", x)?;
        self.draw_distance_info(canvas, &mut y, 5.0, &self.distances.right, "Right", x)?;
        self.draw_distance_info(canvas, &mut y, 5.0, &self.distances.bottom_right, "Bottom right", x)?;
        self.draw_distance_info(canvas, &mut y, 5.0, &self.distances.bottom, "Bottom", x)?;
        self.draw_distance_info(canvas, &mut y, 5.0, &self.distances.bottom_left, "Bottom left", x)?;
        self.draw_distance_info(canvas, &mut y, 5.0, &self.distances.left, "Left", x)?;
        self.draw_distance_info(canvas, &mut y, 5.0, &self.distances.top_left, "Top left", x)?;

        Ok(())
    }

    fn draw_distance_info(&self, canvas: &mut Canvas, start_y: &mut f32, space_between: f32, distance: &DistanceInfo, name: &str, x: f32) -> Result<(), GameError> {
        let mut text = graphics::Text::new(format!("{} to wall: {}", name, distance.distance_to_wall));
        let size = 20.0;
        text.set_scale(size);

        canvas.draw(
            &text,
            DrawParam::new()
                .dest(Vec2::new(x, *start_y))
                .color(Color::from_rgb(0, 0, 0))
        );

        text = graphics::Text::new(format!("{} to apple: {}", name, distance.distance_to_apple));
        text.set_scale(size);
        *start_y += size + space_between;

        canvas.draw(
            &text,
            DrawParam::new()
                .dest(Vec2::new(x, *start_y))
                .color(Color::from_rgb(0, 0, 0))
        );

        text = graphics::Text::new(format!("{} to body: {}", name, distance.distance_to_body));
        text.set_scale(size);
        *start_y += size + space_between;

        canvas.draw(
            &text,
            DrawParam::new()
                .dest(Vec2::new(x, *start_y))
                .color(Color::from_rgb(0, 0, 0))
        );

        *start_y += size + 2.0 * space_between;

        Ok(())
    }
}

pub fn play_game_with_ml(neural_network_options: NeuralNetworkOptions, weights: Vec<Vec<f64>>) -> GameResult {
    let (ctx, events_loop) = ContextBuilder::new("Snake game", "Siemano")
        .window_setup(WindowSetup::default().title("Snake game"))
        .window_mode(WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;

    let state = MLSnakeGameState::new(neural_network_options, weights);

    event::run(ctx, events_loop, state);
}