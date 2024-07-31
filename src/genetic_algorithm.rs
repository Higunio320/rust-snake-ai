use rand::{Rng, thread_rng};
use itertools::Itertools;

#[derive(Clone, PartialEq)]
struct Individual {
    chromosomes: Vec<f64>,
    evaluation: f64
}

struct Population {
    individuals: Vec<Individual>,
    crossing_prob: f64,
    mutation_prob: f64,
    mutation_range: f64,
    n_of_generations: u8,
}

impl Individual {
    fn new(number_of_chromosomes: usize, min_val: f64, max_val: f64) -> Self {
        let mut chromosomes = Vec::with_capacity(number_of_chromosomes);

        let mut rng = thread_rng();

        for _ in 0..number_of_chromosomes {
            chromosomes.push(rng.gen_range(min_val..max_val));
        }

        Individual {chromosomes, evaluation: 0.0}
    }

    fn cross(mut self, mut other: Self) -> (Self, Self) {
        let mut rng = thread_rng();

        let point = rng.gen_range(1..(self.chromosomes.len()-1));

        let mut new_chromosomes_1 = Vec::with_capacity(self.chromosomes.len());
        let mut new_chromosomes_2 = Vec::with_capacity(self.chromosomes.len());

        let remaining_self = &mut self.chromosomes.split_off(point);
        let remaining_other = &mut other.chromosomes.split_off(point);

        new_chromosomes_1.append(&mut self.chromosomes);
        new_chromosomes_2.append(&mut other.chromosomes);

        new_chromosomes_1.append(remaining_other);
        new_chromosomes_2.append(remaining_self);

        return (
            Individual {chromosomes: new_chromosomes_1, evaluation: 0.0},
            Individual {chromosomes: new_chromosomes_2, evaluation: 0.0}
        )
    }

    fn mutate(&mut self, mutation_range: &f64, mutation_prob: &f64) {
        let mut rng = thread_rng();

        self.chromosomes.iter_mut()
            .for_each(|item| {
                if rng.gen_range(0.0..=1.0) < *mutation_prob {
                    *item += rng.gen_range(-(*mutation_range)..=(*mutation_range));
                }
            })
    }

    fn evaluate<F>(&mut self, func: &F)
        where
            F: Fn(&Vec<f64>) -> f64 {
        self.evaluation = func(&self.chromosomes);
    }
}

impl Population {
    pub fn new<F>(population_size: usize, number_of_chromosomes: usize, gen_min_val: f64, gen_max_val: f64,
    crossing_prob: f64, mutation_prob: f64, mutation_range: f64, n_of_generations: u8, evaluation_function: F) -> Self
        where
            F: Fn(&Vec<f64>) -> f64 {
        let mut individuals = Vec::with_capacity(population_size);

        for _ in 0..population_size {
            let mut individual = Individual::new(number_of_chromosomes, gen_min_val, gen_max_val);
            individual.evaluate(&evaluation_function);
            individuals.push(individual);
        }

        Population {individuals, crossing_prob, mutation_prob, mutation_range, n_of_generations}
    }

    pub fn generate_new_population<F>(&mut self, evaluation_function: F)
        where
            F: Fn(&Vec<f64>) -> f64 {
        let new_population = self.selection();

        let mut new_population = self.cross_population(new_population);

        new_population.iter_mut()
            .for_each(|individual| individual.mutate(&self.mutation_range, &self.mutation_prob));


        for individual in self.individuals.iter_mut() {
            individual.evaluate(&evaluation_function)
        }
    }

    fn selection(&self) -> Vec<Individual> {
        let evaluation_sum: f64 = self.individuals.iter()
            .map(|individual| individual.evaluation)
            .sum();

        let probabilities: Vec<f64> = self.individuals.iter()
            .map(|individual| individual.evaluation / evaluation_sum)
            .collect();

        let mut sum = 0.0;

        let mut accumulated_probabilities = Vec::with_capacity(self.individuals.len());

        for probability in probabilities.iter() {
            sum += probability;
            accumulated_probabilities.push(sum);
        }

        let mut rng = thread_rng();

        let mut new_population = Vec::with_capacity(self.individuals.len());

        for _ in 0..self.individuals.len() {
            let r: f64 = rng.gen_range(0.0..=1.0);
            let mut index = 0;

            while index < self.individuals.len() && accumulated_probabilities[index] > r {
                index += 1;
            }

            new_population.push(self.individuals[index].clone());
        }

        new_population
    }

    fn cross_population(&self, population: Vec<Individual>) -> Vec<Individual> {
        let mut rng = thread_rng();

        let mut individuals_to_cross = Vec::with_capacity(population.len());
        let mut individuals_not_to_cross = Vec::with_capacity(population.len());

        for index in 0..population.len() {
            if rng.gen_range(0.0..=1.0) < self.crossing_prob {
                individuals_to_cross.push(population[index].clone());
            } else {
                individuals_not_to_cross.push(population[index].clone());
            }
        }

        let mut crossed_individuals: Vec<Individual> = individuals_to_cross.into_iter()
            .tuples()
            .map(|(first, second)| first.cross(second))
            .flat_map(|(first, second)| vec![first, second])
            .collect();

        individuals_not_to_cross.append(&mut crossed_individuals);

        individuals_not_to_cross
    }
}


