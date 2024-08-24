use std::cmp::{max_by};
use rand::{Rng, thread_rng};
use crate::ai::genetic_algorithm::{Population, PopulationOptions};
use crate::ai::neural_network::NeuralNetwork;
use crate::ai::neural_network_utils::NeuralNetworkOptions;
use crate::snake::snake_game::{Ate, Direction, DistanceInfo, Food, Position, Snake};
use crate::visualisation::game_constants::{MAX_DISTANCE, MAX_X_DISTANCE, MAX_Y_DISTANCE, GRID_SIZE};
use crate::visualisation::ml_game::play_game_with_ml;

pub const FIRST_LAYER_SIZE: usize = 32;
pub const OUTPUT_LAYER_SIZE: usize = 3;

const MAX_STEPS_WITHOUT_APPLE: f64 = 150.0;

const POINTS_BASE: f64 = 2.0;

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

        println!("Best of the best: {:?}", populations[populations.len()-1]);

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
    let mut steps: f64 = 0.0;
    let mut steps_without_apple = 0.0;
    let mut score: f64 = 0.0;

    while !game_over && steps_without_apple < MAX_STEPS_WITHOUT_APPLE {
        steps += 1.0;
        steps_without_apple += 1.0;

        let output = neural_network.get_output(input).unwrap();

        let move_dir = interpret_network_output(&output);

        snake.move_in_dir(move_dir);

        snake.update_state(&food);

        if let Some(ate) = snake.get_ate() {
            match ate {
                Ate::Food => {
                    food = generate_new_food(&snake);
                    score += 1.0;
                    steps_without_apple = 0.0;
                },
                Ate::Itself | Ate::Border => game_over = true
            }
        }

        input = generate_network_input(&snake, &food);
    }

    max_by(steps + POINTS_BASE.powf(score) + score.powf(2.1)*500.0 - (score.powf(1.2) * (steps * 0.25).powf(1.3)), 0.0, |a, b| a.total_cmp(b))
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

    add_distance_to_input(distances.top, &mut input, MAX_Y_DISTANCE);
    add_distance_to_input(distances.right, &mut input, MAX_X_DISTANCE);
    add_distance_to_input(distances.bottom, &mut input, MAX_Y_DISTANCE);
    add_distance_to_input(distances.left, &mut input, MAX_X_DISTANCE);

    add_distance_to_input(distances.top_right, &mut input, *MAX_DISTANCE);
    add_distance_to_input(distances.bottom_right, &mut input, *MAX_DISTANCE);
    add_distance_to_input(distances.bottom_left, &mut input, *MAX_DISTANCE);
    add_distance_to_input(distances.top_left, &mut input, *MAX_DISTANCE);

    match snake.get_current_direction() {
        Direction::UP => input.append(&mut vec![1.0, 0.0, 0.0, 0.0]),
        Direction::RIGHT => input.append(&mut vec![0.0, 1.0, 0.0, 0.0]),
        Direction::DOWN => input.append(&mut vec![0.0, 0.0, 1.0, 0.0]),
        Direction::LEFT => input.append(&mut vec![0.0, 0.0, 0.0, 1.0]),
    }

    match snake.get_tail_direction() {
        Direction::UP => input.append(&mut vec![1.0, 0.0, 0.0, 0.0]),
        Direction::RIGHT => input.append(&mut vec![0.0, 1.0, 0.0, 0.0]),
        Direction::DOWN => input.append(&mut vec![0.0, 0.0, 1.0, 0.0]),
        Direction::LEFT => input.append(&mut vec![0.0, 0.0, 0.0, 1.0]),
    }

    input
}

fn add_distance_to_input(distance: DistanceInfo, input: &mut Vec<f64>, max: f64) {
    input.push(distance.distance_to_wall / max);
    input.push(distance.distance_to_apple);
    input.push(distance.distance_to_body);
}

pub enum Move {
    FORWARD,
    LEFT,
    RIGHT
}

pub fn interpret_network_output(output: &Vec<f64>) -> Direction {
    let mut max = 0.0;
    let mut index = 0;

    for (i, val) in output.iter().enumerate() {
        if *val > max {
            max = *val;
            index = i;
        }
    }

    if index == 0 {
        Direction::UP
    } else if index == 1 {
        Direction::RIGHT
    } else if index == 2 {
        Direction::DOWN
    } else {
        Direction::LEFT
    }
}

