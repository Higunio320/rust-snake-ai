use crate::game::play_game;
use crate::genetic_algorithm::PopulationOptions;
use crate::neural_network::{NeuralNetworkOptions, ReLU, Softmax};
use crate::snake_trainer::{FIRST_LAYER_SIZE, MLSnakeOptions, SnakeTrainer};

mod snake_game;
mod game;
mod neural_network;
mod genetic_algorithm;
mod snake_trainer;
mod ml_game;

fn main() {
    let population_options = PopulationOptions::new(
        500,
        FIRST_LAYER_SIZE * 20 + 20 * 12 + 12 * 4,
        -1.0,
        1.0,
        0.9,
        0.05,
        0.3,
        2000
    );

    let neural_network_options = NeuralNetworkOptions::new(
        vec![FIRST_LAYER_SIZE as u16, 20, 12, 4],
        vec![Box::new(ReLU), Box::new(ReLU), Box::new(Softmax)]
    );

    SnakeTrainer::train(MLSnakeOptions::new(population_options, neural_network_options));
}
