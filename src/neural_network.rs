use std::fmt::{Debug};
use rand::{Rng, thread_rng};

pub(crate) struct NeuralNetwork {
    layers_weights: Vec<f64>,
    layers_functions: Vec<Box<dyn Function>>,
    layers_sizes_vec: Vec<u16>
}


pub trait Function: Debug + FunctionClone {
    fn apply(&self, input: &mut Vec<f64>);
}

pub trait FunctionClone {
    fn clone_box(&self) -> Box<dyn Function>;
}

impl<T> FunctionClone for T
where
    T: 'static + Function + Clone,
{
    fn clone_box(&self) -> Box<dyn Function> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Function> {
    fn clone(&self) -> Box<dyn Function> {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct ReLU;

impl Function for ReLU {
    fn apply(&self, input: &mut Vec<f64>) {
        for number in input.iter_mut() {
            if *number < 0.0 {
                *number = 0.0;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Softmax;

impl Function for Softmax {
    fn apply(&self, input: &mut Vec<f64>) {
        let (exps, sum): (Vec<f64>, f64) = input.iter()
            .map(|number| number.exp())
            .fold((Vec::with_capacity(input.len()), 0.0), |(mut exps, sum), exp| {
                exps.push(exp);
                (exps, sum + exp)
            });

        for(i, number) in input.iter_mut().enumerate() {
            *number = exps[i] / sum;
        }
    }
}

#[derive(Clone)]
pub struct NeuralNetworkOptions {
    layers_sizes_vec: Vec<u16>,
    layers_functions: Vec<Box<dyn Function>>
}

impl NeuralNetworkOptions {
    pub fn new(layers_sizes_vec: Vec<u16>, layers_functions: Vec<Box<dyn Function>>) -> Self {
        NeuralNetworkOptions {
            layers_sizes_vec,
            layers_functions
        }
    }
}

impl NeuralNetwork {
    pub fn new(options: NeuralNetworkOptions) -> Result<Self, String> {
        let layers_sizes_vec = options.layers_sizes_vec;
        let layers_functions = options.layers_functions;

        if layers_functions.len() != layers_sizes_vec.len() - 1 {
            return Err(format!("Functions len: {} must be layers len: {} - 1", layers_functions.len(),
                               layers_sizes_vec.len()))
        }

        let mut rng = thread_rng();

        let capacity: usize = layers_sizes_vec.windows(2)
            .map(|window| (window[0] * window[1]) as usize)
            .sum();

        let mut layers_weights = Vec::with_capacity(capacity);

        let mut iterator = layers_sizes_vec.iter();
        iterator.next();

        for (i, size) in iterator.enumerate() {
            for _neuron in 0..(*size).into() {
                for _weight in 0..layers_sizes_vec[i] {
                    layers_weights.push(rng.gen_range(-1.0..=1.0));
                }
            }
        }

        Ok(NeuralNetwork {layers_weights, layers_functions, layers_sizes_vec})
    }

    pub fn new_with_weights(layers_weights: Vec<f64>, neural_network_options: NeuralNetworkOptions) -> Result<Self, String> {
        let layers_sizes_vec = neural_network_options.layers_sizes_vec;
        let layers_functions = neural_network_options.layers_functions;

        if layers_functions.len() != layers_sizes_vec.len() - 1 {
            return Err(format!("Functions len: {} must be layers len: {} - 1", layers_functions.len(),
                               layers_sizes_vec.len()))
        }

        let capacity: usize = layers_sizes_vec.windows(2)
            .map(|window| (window[0] * window[1]) as usize)
            .sum();

        if capacity != layers_weights.len() {
            return Err(format!("Weights len: {} and layers sizes: {:?} don't match. Expected length: {}",
                               layers_weights.len(), layers_sizes_vec, capacity))
        }

        Ok(NeuralNetwork {layers_weights, layers_functions, layers_sizes_vec})
    }

    pub fn get_output(&self, input: Vec<f64>) -> Result<Vec<f64>, String> {
        //unsafe indexing
        if input.len() != self.layers_sizes_vec[0] as usize {
            return Err(format!("Input len: {} doesn't match network input len: {}", input.len(),
            self.layers_sizes_vec[0]))
        }

        let mut output = input;

        let mut layer_sizes = self.layers_sizes_vec.iter();
        let mut beginning_index = 0;
        let mut previous_layer_length = layer_sizes.next().unwrap_or(&0);

        for(i, layer_size) in layer_sizes.enumerate() {
            output = calculate_output_from_layer(output, &self.layers_weights[beginning_index..beginning_index+((*layer_size * *previous_layer_length) as usize)], &self.layers_functions[i]);
            beginning_index += (*layer_size * *previous_layer_length) as usize;
            previous_layer_length = layer_size;
        }

        Ok(output)
    }

    pub fn update_weights(&mut self, new_weights: Vec<f64>) {
        self.layers_weights = new_weights;
    }
}

fn calculate_output_from_layer(input: Vec<f64>, layer: &[f64], function: &Box<dyn Function>) -> Vec<f64> {
    let mut output = layer.chunks(input.len())
        .map(|item| item.iter()
            .zip(input.iter())
            .map(|(weight, input)| input * weight)
            .sum())
        .collect();

    function.apply(&mut output);

    output
}

#[cfg(test)]
mod test {
    use crate::neural_network::{Function, NeuralNetwork, NeuralNetworkOptions, ReLU, Softmax};

    #[test]
    pub fn new_neural_network_constructs_correct_network() {
        //given
        let layers_sizes_vec = vec![4, 3, 2];
        let layers_functions: Vec<Box<dyn Function>> = vec![Box::new(ReLU{}), Box::new(Softmax{})];

        let options = NeuralNetworkOptions {
            layers_sizes_vec: layers_sizes_vec.clone(),
            layers_functions
        };

        //when
        let neural_network = match NeuralNetwork::new(options) {
            Ok(network) => network,
            Err(_) => {
                assert!(false, "Function should return Ok");
                panic!()
            }
        };

        //then
        let expected_layers_sizes_len = 18;

        assert_eq!(neural_network.layers_weights.len(), expected_layers_sizes_len,
                   "There should be {} weights", expected_layers_sizes_len);
        assert_eq!(neural_network.layers_sizes_vec, layers_sizes_vec,
                   "The sizes should be the same");

        neural_network.layers_weights.iter()
            .for_each(|weight| assert!(*weight <= 1.0 && *weight >= -1.0,
                                       "Every weight should be between -1.0 and 1.0"));
    }

    #[test]
    pub fn new_neural_network_should_return_error_on_incorrect_options() {
        //given
        let layers_sizes_vec = vec![4, 3, 2];
        let layers_functions: Vec<Box<dyn Function>> = vec![Box::new(ReLU{})];

        let options = NeuralNetworkOptions {
            layers_sizes_vec: layers_sizes_vec.clone(),
            layers_functions
        };

        //when-then
        assert!(NeuralNetwork::new(options).is_err(), "There should be an error");
    }

    #[test]
    pub fn new_with_weights_should_construct_correct_neural_network() {
        //given
        let layers_sizes_vec = vec![4, 3, 2];
        let layers_functions: Vec<Box<dyn Function>> = vec![Box::new(ReLU {}), Box::new(Softmax {})];
        let layers_weights = vec![1.0; 18];

        let options = NeuralNetworkOptions {
            layers_sizes_vec: layers_sizes_vec.clone(),
            layers_functions
        };

        //when
        let neural_network = match NeuralNetwork::new_with_weights(layers_weights.clone(), options) {
            Ok(network) => network,
            Err(_) => {
                assert!(false, "Function should return Ok");
                panic!()
            }
        };

        //then
        assert_eq!(neural_network.layers_weights, layers_weights,
                   "Layers weights should be the same");
        assert_eq!(neural_network.layers_sizes_vec, layers_sizes_vec,
                   "The sizes should be the same")
    }

    #[test]
    pub fn new_with_weights_should_return_err_on_incorrect_options() {
        //given
        let layers_sizes_vec = vec![4, 3, 2];
        let layers_functions: Vec<Box<dyn Function>> = vec![Box::new(ReLU{})];
        let layers_weights = vec![1.0; 18];

        let options = NeuralNetworkOptions {
            layers_sizes_vec: layers_sizes_vec.clone(),
            layers_functions
        };

        //when-then
        assert!(NeuralNetwork::new_with_weights(layers_weights, options).is_err(), "There should be an error");
    }

    #[test]
    pub fn new_with_weights_should_return_err_on_incorrect_layers_weights() {
        //given
        let layers_sizes_vec = vec![4, 3, 2];
        let layers_functions: Vec<Box<dyn Function>> = vec![Box::new(ReLU {}), Box::new(Softmax {})];
        let layers_weights = vec![1.0; 20];

        let options = NeuralNetworkOptions {
            layers_sizes_vec: layers_sizes_vec.clone(),
            layers_functions
        };

        //when-then
        assert!(NeuralNetwork::new_with_weights(layers_weights, options).is_err(), "There should be an error");
    }

    #[test]
    pub fn get_output_should_calculate_correctly() {
        //given
        let layers_sizes_vec = vec![3, 2, 2];
        let layers_functions: Vec<Box<dyn Function>> = vec![Box::new(ReLU {}), Box::new(Softmax {})];
        let layers_weights = vec![1.0, 2.0, 0.5, 0.5, 1.0, 2.0, 1.0, 1.0, 0.5, 1.0];

        let options = NeuralNetworkOptions {
            layers_sizes_vec: layers_sizes_vec.clone(),
            layers_functions
        };

        let neural_network = match NeuralNetwork::new_with_weights(layers_weights.clone(), options) {
            Ok(network) => network,
            Err(_) => {
                assert!(false, "Function should return Ok");
                panic!()
            }
        };

        let input = vec![1.0, 2.0, 3.0];

        //when
        let expected_output = vec![0.96267_f64, 0.03732_f64];

        let output = match neural_network.get_output(input) {
            Ok(output) => output,
            Err(_) => {
                assert!(false, "Function should return Ok");
                panic!()
            }
        };

        expected_output.iter()
            .zip(output.iter())
            .for_each(|(a, b)| assert_equal_with_error(*b, *a, 0.0005));
    }

    fn assert_equal_with_error(actual: f64, expected: f64, error: f64) {
        println!("{} {}", actual, expected);
        assert!(actual >= expected - error && actual <= expected + error,
        "{actual} should be in {} - {}", expected - error, expected + error);
    }
}
