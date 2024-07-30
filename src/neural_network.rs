use std::fmt::format;
use rand::{Rng, thread_rng};

struct NeuralNetwork {
    layers_weights: Vec<Vec<Vec<f64>>>,
    layers_functions: Vec<Box<dyn Function>>
}

trait Function {
    fn apply(&self, input: &mut Vec<f64>);
}

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

impl NeuralNetwork {
    pub fn new(layer_sizes_vec: Vec<u8>, layers_functions: Vec<Box<dyn Function>>) -> Result<Self, String> {
        if layers_functions.len() != layer_sizes_vec.len() - 1 {
            return Err(format!("Functions len: {} must be layers len: {} - 1", layers_functions.len(),
                               layer_sizes_vec.len()))
        }

        let mut rng = thread_rng();

        let mut layers_weights = Vec::with_capacity(layer_sizes_vec.len() - 1);

        let mut iterator = layer_sizes_vec.iter();
        iterator.next();

        for (i, size) in iterator.enumerate() {
            let mut layer = Vec::with_capacity(*size as usize);

            for _neuron in 0..(*size).into() {
                let mut neuron = Vec::with_capacity(layer_sizes_vec[i].into());

                for _weight in 0..layer_sizes_vec[i] {
                    neuron.push(rng.gen_range(-1.0..1.0));
                }

                layer.push(neuron);
            }

            layers_weights.push(layer);
        }

        Ok(NeuralNetwork {layers_weights, layers_functions})
    }

    pub fn new_with_weights(layers_weights: Vec<Vec<Vec<f64>>>, layers_functions: Vec<Box<dyn Function>>) -> Self {
        NeuralNetwork {layers_weights, layers_functions}
    }

    pub fn get_output(&self, input: Vec<f64>) -> Result<Vec<f64>, String> {
        //unsafe indexing
        if input.len() != self.layers_weights[0][0].len() {
            return Err(format!("Input len: {} doesn't match network input len: {}", input.len(),
            self.layers_weights[0][0].len()))
        }

        let mut output = input;

        for (i, layer) in self.layers_weights.iter().enumerate() {
            output = calculate_output_from_layer(output, layer, &self.layers_functions[i]);
        }

        Ok(output)
    }

}

fn calculate_output_from_layer(input: Vec<f64>, layer: &Vec<Vec<f64>>, function: &Box<dyn Function>) -> Vec<f64> {
    let mut output = Vec::with_capacity(layer.len());

    for neuron in layer.iter() {
        let sum= neuron.iter()
                .zip(input.iter())
                .map(|(input, weight)| input * weight)
                .sum();

        output.push(sum)
    }

    function.apply(&mut output);

    output
}
