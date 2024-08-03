use std::sync::{Arc, Mutex};
use ggez::event::EventHandler;
use ggez::{Context, ContextBuilder, event, GameError, GameResult, graphics};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::glam::Vec2;
use ggez::graphics::{Canvas, Color, DrawParam};
use rand::prelude::ThreadRng;
use rand::thread_rng;
use crate::game::{FPS, SCREEN_SIZE};
use crate::neural_network::{NeuralNetwork, NeuralNetworkOptions};
use crate::snake_game::{Ate, Food, Snake};
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
    max_steps: u8,
    current_steps: u8
}

impl MLSnakeGameState {
    fn new(neural_network_options: NeuralNetworkOptions, weights: Vec<Vec<f64>>) -> Self {
        let snake_pos = generate_random_position();

        let rng = thread_rng();

        let current_game_index: usize = 0;

        let neural_network = NeuralNetwork::new_with_weights(weights[0].clone(), neural_network_options).unwrap();

        let snake = Snake::new(snake_pos);

        let food = generate_new_food(&snake);

        let current_score = 0_u16;

        let max_steps = 30_u8;

        let current_steps = 0_u8;

        MLSnakeGameState {
            snake,
            food,
            neural_network,
            rng,
            game_over: false,
            current_game_index,
            weights,
            current_score,
            max_steps,
            current_steps
        }
    }
}

impl EventHandler<GameError> for MLSnakeGameState {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        while ctx.time.check_update_time(FPS) {
            if !self.game_over && self.current_steps < self.max_steps {
                self.current_steps += 1;

                let input = generate_network_input(&self.snake, &self.food);

                let output = self.neural_network.get_output(input).unwrap();

                let move_dir = interpret_network_output(&output);

                self.snake.move_in_dir_with_move(move_dir);

                self.snake.update_state(&self.food);

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
                    self.current_steps = 0;

                    let snake_pos = generate_random_position();

                    let snake = Snake::new(snake_pos);

                    let food = generate_new_food(&snake);

                    self.neural_network.update_weights(self.weights[self.current_game_index].clone());

                    self.current_game_index += 1;

                    self.snake = snake;

                    self.food = food;

                    self.game_over = false;
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

        let mut text = graphics::Text::new(format!("Current gen: {}, current score: {}",
                                                   self.current_game_index + 1, self.current_score));

        text.set_scale(28.);

        canvas.draw(
            &text,
            DrawParam::new()
                .dest(Vec2::new(5.0, 5.0))
                .color(Color::from_rgb(0, 0, 0))
        );

        canvas.finish(ctx)?;

        ggez::timer::yield_now();

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