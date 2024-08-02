use crate::genetic_algorithm::{Population, PopulationOptions};
use crate::neural_network::{NeuralNetwork, NeuralNetworkOptions};

pub struct MLSnakeOptions {
    genetic_algorithm_options: PopulationOptions,
    neural_network_options: NeuralNetworkOptions
}

pub struct SnakeTrainer {
    population: Population,
    neural_network_options: NeuralNetworkOptions
}

