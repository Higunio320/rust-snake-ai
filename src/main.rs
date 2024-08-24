use crate::ai::genetic_algorithm::PopulationOptions;
use crate::ai::neural_network_utils::{NeuralNetworkOptions, ReLU, Softmax};
use crate::ai::snake_trainer::{MLSnakeOptions, SnakeTrainer, FIRST_LAYER_SIZE};

mod visualisation;
mod ai;
mod snake;

fn main() {
    let population_options = PopulationOptions::new(
        500,
        FIRST_LAYER_SIZE * 20 + 20 * 12 + 12 * 4,
        -1.0,
        1.0,
        0.9,
        0.3,
        0.3,
        2000
    );

    let neural_network_options = NeuralNetworkOptions::new(
        vec![FIRST_LAYER_SIZE as u16, 20, 12, 4],
        vec![Box::new(ReLU), Box::new(ReLU), Box::new(Softmax)]
    );

    SnakeTrainer::train(MLSnakeOptions::new(population_options, neural_network_options));
}
