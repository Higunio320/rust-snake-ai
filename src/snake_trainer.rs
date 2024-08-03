use std::sync::{Arc, Mutex};
use std::thread;
use rand::{Rng, thread_rng};
use crate::game::{GRID_SIZE, MAX_DISTANCE};
use crate::genetic_algorithm::{Population, PopulationOptions};
use crate::ml_game::play_game_with_ml;
use crate::neural_network::{NeuralNetwork, NeuralNetworkOptions};
use crate::snake_game::{Ate, Direction, DistanceInfo, Food, Position, Snake};

pub const FIRST_LAYER_SIZE: usize = 28;
pub const OUTPUT_LAYER_SIZE: usize = 3;

const MAX_STEPS: u16 = 10000;

const POINTS_BASE: u32 = 2;

pub struct MLSnakeOptions {
    genetic_algorithm_options: PopulationOptions,
    neural_network_options: NeuralNetworkOptions
}

impl MLSnakeOptions {
    pub fn new(genetic_algorithm_options: PopulationOptions,
               neural_network_options: NeuralNetworkOptions) -> Self {
        MLSnakeOptions {
            genetic_algorithm_options,
            neural_network_options
        }
    }
}

pub struct SnakeTrainer;

impl SnakeTrainer {
    pub fn train(options: MLSnakeOptions) {
        let n_of_generations = options.genetic_algorithm_options.n_of_generations;
        let mut population = Population::new(options.genetic_algorithm_options, evaluate, &options.neural_network_options);

        let mut populations = Vec::with_capacity((n_of_generations + 1) as usize);

        for i in 0..n_of_generations {
            println!("Generation: {}", i+1);
            population.generate_new_population(evaluate, &options.neural_network_options);
            println!("Best score: {}", population.get_best_score());
            populations.push(population.get_best_chromosomes());
        }

        play_game_with_ml(options.neural_network_options, populations).unwrap()
    }
}

pub fn evaluate(chromosomes: &Vec<f64>, neural_network_options: &NeuralNetworkOptions) -> f64 {
    let neural_network = NeuralNetwork::new_with_weights(chromosomes.clone(),
                                                         (*neural_network_options).clone()).unwrap();

    let snake_pos = generate_random_position_with_distance(2);

    let mut snake = Snake::new(snake_pos);

    let mut food = generate_new_food(&snake);

    let mut input = generate_network_input(&snake, &food);

    let mut game_over = false;
    let mut steps: u16 = 0;
    let mut score: u16 = 0;

    while !game_over && steps < MAX_STEPS {
        steps += 1;

        let output = neural_network.get_output(input).unwrap();

        let move_dir = interpret_network_output(&output);

        snake.move_in_dir_with_move(move_dir);

        snake.update_state(&food);

        if let Some(ate) = snake.get_ate() {
            match ate {
                Ate::Food => {
                    food = generate_new_food(&snake);
                    score += 1;
                },
                Ate::Itself | Ate::Border => game_over = true
            }
        }

        input = generate_network_input(&snake, &food);
    }

    POINTS_BASE.pow(score as u32) as f64
}

pub fn generate_random_position() -> Position {
    let mut rng = thread_rng();

    Position::new(rng.gen_range(0..GRID_SIZE.0), rng.gen_range(0..GRID_SIZE.1))
}

fn generate_random_position_with_distance(distance_from_walls: i16) -> Position {
    let mut rng = thread_rng();

    Position::new(rng.gen_range(0+distance_from_walls..GRID_SIZE.0-distance_from_walls),
                  rng.gen_range(0+distance_from_walls..GRID_SIZE.1-distance_from_walls))
}

pub fn generate_new_food(snake: &Snake) -> Food {
    let mut position = generate_random_position();

    while snake.is_in_position(position) {
        position = generate_random_position();
    }

    Food::new(position)
}

pub fn generate_network_input(snake: &Snake, food: &Food) -> Vec<f64> {
    let distances = snake.get_distances(food);

    let mut input = Vec::with_capacity(FIRST_LAYER_SIZE);

    add_distance_to_input(distances.top, &mut input);
    add_distance_to_input(distances.right, &mut input);
    add_distance_to_input(distances.bottom, &mut input);
    add_distance_to_input(distances.left, &mut input);

    add_distance_to_input(distances.top_right, &mut input);
    add_distance_to_input(distances.bottom_right, &mut input);
    add_distance_to_input(distances.bottom_left, &mut input);
    add_distance_to_input(distances.top_left, &mut input);

    match snake.get_current_direction() {
        Direction::UP => input.append(&mut vec![1.0, 0.0, 0.0, 0.0]),
        Direction::RIGHT => input.append(&mut vec![0.0, 1.0, 0.0, 0.0]),
        Direction::DOWN => input.append(&mut vec![0.0, 0.0, 1.0, 0.0]),
        Direction::LEFT => input.append(&mut vec![0.0, 0.0, 0.0, 1.0]),
    }

    input
}

fn add_distance_to_input(distance: DistanceInfo, input: &mut Vec<f64>) {
    input.push(distance.distance_to_wall / *MAX_DISTANCE);

    if distance.is_there_an_apple {
        input.push(1.0);
    } else {
        input.push(0.0);
    }

    if distance.is_there_snake {
        input.push(1.0);
    } else {
        input.push(0.0);
    }
}

pub enum Move {
    FORWARD,
    LEFT,
    RIGHT
}

pub fn interpret_network_output(output: &Vec<f64>) -> Move {
    if output[0] > 0.33 {
        Move::LEFT
    } else if output[1] > 0.33 {
        Move::FORWARD
    } else {
        Move::RIGHT
    }
}

