use std::fmt::Debug;

pub trait Function: Debug + FunctionClone + Sync {
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
    pub layers_sizes_vec: Vec<u16>,
    pub layers_functions: Vec<Box<dyn Function>>
}

impl NeuralNetworkOptions {
    pub fn new(layers_sizes_vec: Vec<u16>, layers_functions: Vec<Box<dyn Function>>) -> Self {
        NeuralNetworkOptions {
            layers_sizes_vec,
            layers_functions
        }
    }
}
