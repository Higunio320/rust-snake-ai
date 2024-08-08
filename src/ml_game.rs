use ggez::event::EventHandler;
use ggez::{Context, ContextBuilder, event, GameError, GameResult, graphics};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::glam::Vec2;
use ggez::graphics::{Canvas, Color, DrawParam};
use ggez::input::keyboard::{KeyCode, KeyInput};
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
    stop: bool
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


        MLSnakeGameState {
            snake,
            food,
            neural_network,
            rng,
            game_over: false,
            current_game_index,
            weights,
            current_score,
            stop: false
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

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeated: bool) -> Result<(), GameError> {
        if let Some(code) = input.keycode {
            match code {
                KeyCode::Right => self.stop = true,
                _ => {}
            }
        };

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