use crate::ascii_generator::AsciiGenerator;
use image::{ImageBuffer, Luma};
use rand::{Rng, thread_rng};
use rayon::prelude::*;
use std::cmp::Ordering;
use std::sync::Arc;

/// Limited character set for ASCII art generation
const ALLOWED_CHARS: &[u8] = b" <>,./?\\|[]{}-_=+AvViIoOxXwWM`~;:'\"!@#$%^&*()8";

/// Represents an individual in the genetic algorithm population
#[derive(Clone, Debug)]
pub struct Individual {
    pub chars: Vec<u8>,
    pub fitness: f64,
}

impl Individual {
    /// Creates a new individual with random ASCII characters
    pub fn new_random(size: usize) -> Self {
        let mut rng = thread_rng();
        let chars: Vec<u8> = (0..size)
            .map(|_| ALLOWED_CHARS[rng.gen_range(0..ALLOWED_CHARS.len())])
            .collect();
        
        Self {
            chars,
            fitness: 0.0,
        }
    }
    
    /// Creates a new individual with a specified initialization character
    /// 95% of characters will be the init_char, 5% will be random
    pub fn new_with_init_char(size: usize, init_char: char) -> Self {
        let mut rng = thread_rng();
        let init_byte = init_char as u8;
        
        // Ensure the init_char is in the allowed character set
        let init_byte = if ALLOWED_CHARS.contains(&init_byte) {
            init_byte
        } else {
            b' ' // Default to space if invalid character
        };
        
        let chars: Vec<u8> = (0..size)
            .map(|_| {
                if rng.gen::<f64>() < 0.05 { // 5% chance for random character
                    ALLOWED_CHARS[rng.gen_range(0..ALLOWED_CHARS.len())]
                } else {
                    init_byte
                }
            })
            .collect();
        
        Self {
            chars,
            fitness: 0.0,
        }
    }
    
    /// Creates a new individual from existing character data
    pub fn new(chars: Vec<u8>) -> Self {
        Self {
            chars,
            fitness: 0.0,
        }
    }
    
    /// Performs uniform crossover with another individual
    pub fn crossover(&self, other: &Individual, crossover_rate: f64) -> (Individual, Individual) {
        let mut rng = thread_rng();
        let mut child1_chars = self.chars.clone();
        let mut child2_chars = other.chars.clone();
        
        for i in 0..self.chars.len().min(other.chars.len()) {
            if rng.gen::<f64>() < crossover_rate {
                child1_chars[i] = other.chars[i];
                child2_chars[i] = self.chars[i];
            }
        }
        
        (Individual::new(child1_chars), Individual::new(child2_chars))
    }
    
    /// Performs mutation on the individual
    pub fn mutate(&mut self, mutation_rate: f64) {
        let mut rng = thread_rng();
        
        for char in &mut self.chars {
            if rng.gen::<f64>() < mutation_rate {
                *char = ALLOWED_CHARS[rng.gen_range(0..ALLOWED_CHARS.len())];
            }
        }
    }
}

/// Main genetic algorithm implementation
pub struct GeneticAlgorithm<'a> {
    population: Vec<Individual>,
    population_size: usize,
    width: u32,
    height: u32,
    ascii_generator: &'a AsciiGenerator,
    target_image: &'a ImageBuffer<Luma<u8>, Vec<u8>>,
    mutation_rate: f64,
    crossover_rate: f64,
    elite_size: usize,
    #[cfg(test)]
    thread_count: usize,
}

impl<'a> GeneticAlgorithm<'a> {
    /// Creates a new genetic algorithm instance
    pub fn new(
        width: u32,
        height: u32,
        population_size: usize,
        ascii_generator: &'a AsciiGenerator,
        target_image: &'a ImageBuffer<Luma<u8>, Vec<u8>>,
        thread_count: usize,
        init_char: Option<char>,
    ) -> Self {
        let individual_size = (width * height) as usize;
        let population: Vec<Individual> = (0..population_size)
            .map(|_| {
                match init_char {
                    Some(ch) => Individual::new_with_init_char(individual_size, ch),
                    None => Individual::new_random(individual_size),
                }
            })
            .collect();
        
        // Set up thread pool for parallel processing
        // Only initialize if not already initialized (for testing compatibility)
        if let Err(e) = rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build_global()
        {
            // Check if the global pool is already initialized, which is fine for tests
            let error_string = format!("{:?}", e);
            if !error_string.contains("GlobalPoolAlreadyInitialized") {
                panic!("Failed to initialize thread pool: {:?}", e);
            }
        }
        
        Self {
            population,
            population_size,
            width,
            height,
            ascii_generator,
            target_image,
            mutation_rate: 0.01,
            crossover_rate: 0.8,
            elite_size: population_size / 10, // Top 10% are elite
            #[cfg(test)]
            thread_count,
        }
    }
    
    /// Runs the genetic algorithm for the specified number of generations
    pub fn evolve(&mut self, generations: u32, verbose: bool) -> Individual {
        for generation in 0..generations {
            self.evaluate_population();
            
            if generation % 10 == 0 {
                let best_fitness = self.population[0].fitness;
                println!("Generation {}: Best fitness = {:.2}%", generation, best_fitness * 100.0);
                
                if verbose {
                    let best_ascii = self.ascii_generator.individual_to_string(&self.population[0], self.width);
                    println!("Current best ASCII art:\n{}\n", best_ascii);
                }
            }
            
            if generation < generations - 1 {
                self.create_new_generation();
            }
        }
        
        self.evaluate_population();
        self.population[0].clone()
    }
    
    /// Evaluates the fitness of all individuals in the population using parallel processing
    fn evaluate_population(&mut self) {
        // Clone chars to avoid borrowing issues and prepare for parallel processing
        let chars_list: Vec<Vec<u8>> = self.population
            .iter()
            .map(|individual| individual.chars.clone())
            .collect();
        
        // Create Arc references for thread-safe sharing
        let ascii_gen = Arc::new(self.ascii_generator);
        let target_img = Arc::new(self.target_image.clone());
        let width = self.width;
        let height = self.height;
        
        // Calculate fitness in parallel
        let fitness_values: Vec<f64> = chars_list
            .par_iter()
            .map(|chars| {
                Self::calculate_fitness_for_chars_static(
                    chars, 
                    &ascii_gen, 
                    &target_img, 
                    width, 
                    height
                )
            })
            .collect();
        
        // Update fitness values
        for (individual, fitness) in self.population.iter_mut().zip(fitness_values.iter()) {
            individual.fitness = *fitness;
        }
        
        // Sort population by fitness (descending)
        self.population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(Ordering::Equal));
    }
    
    /// Calculates fitness as percentage of matching pixels between ASCII art and target image
    #[cfg(test)]
    fn calculate_fitness(&self, individual: &Individual) -> f64 {
        self.calculate_fitness_for_chars(&individual.chars)
    }
    
    /// Calculates fitness for a given character array
    #[cfg(test)]
    fn calculate_fitness_for_chars(&self, chars: &[u8]) -> f64 {
        Self::calculate_fitness_for_chars_static(
            chars, 
            &Arc::new(self.ascii_generator), 
            &Arc::new(self.target_image.clone()), 
            self.width, 
            self.height
        )
    }
    
    /// Static version of fitness calculation for parallel processing
    fn calculate_fitness_for_chars_static(
        chars: &[u8], 
        ascii_generator: &Arc<&AsciiGenerator>, 
        target_image: &Arc<ImageBuffer<Luma<u8>, Vec<u8>>>, 
        width: u32, 
        height: u32
    ) -> f64 {
        let ascii_image = ascii_generator.generate_ascii_image(chars, width, height);
        
        let mut matches = 0;
        let mut total = 0;
        
        let min_width = ascii_image.width().min(target_image.width());
        let min_height = ascii_image.height().min(target_image.height());
        
        for y in 0..min_height {
            for x in 0..min_width {
                let ascii_pixel = ascii_image.get_pixel(x, y)[0];
                let target_pixel = target_image.get_pixel(x, y)[0];
                
                // Use a tolerance for pixel matching
                let diff = (ascii_pixel as i32 - target_pixel as i32).abs();
                if diff < 30 { // Tolerance of 30 out of 255
                    matches += 1;
                }
                total += 1;
            }
        }
        
        if total == 0 {
            0.0
        } else {
            matches as f64 / total as f64
        }
    }
    
    /// Creates a new generation using selection, crossover, and mutation
    fn create_new_generation(&mut self) {
        let mut new_population = Vec::with_capacity(self.population_size);
        
        // Keep elite individuals
        for i in 0..self.elite_size {
            new_population.push(self.population[i].clone());
        }
        
        // Generate offspring to fill the rest of the population
        while new_population.len() < self.population_size {
            let parent1 = self.tournament_selection();
            let parent2 = self.tournament_selection();
            
            let (mut child1, mut child2) = parent1.crossover(&parent2, self.crossover_rate);
            
            child1.mutate(self.mutation_rate);
            child2.mutate(self.mutation_rate);
            
            new_population.push(child1);
            if new_population.len() < self.population_size {
                new_population.push(child2);
            }
        }
        
        self.population = new_population;
    }
    
    /// Performs tournament selection to choose a parent for reproduction
    fn tournament_selection(&self) -> Individual {
        let mut rng = thread_rng();
        let tournament_size = 3;
        
        let mut best_individual = &self.population[rng.gen_range(0..self.population.len())];
        
        for _ in 1..tournament_size {
            let candidate = &self.population[rng.gen_range(0..self.population.len())];
            if candidate.fitness > best_individual.fitness {
                best_individual = candidate;
            }
        }
        
        best_individual.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageBuffer;

    fn create_test_ascii_generator() -> AsciiGenerator {
        AsciiGenerator::new()
    }
    
    fn create_test_target_image() -> ImageBuffer<Luma<u8>, Vec<u8>> {
        ImageBuffer::new(20, 20)
    }

    #[test]
    fn test_individual_creation() {
        let individual = Individual::new_random(100);
        assert_eq!(individual.chars.len(), 100);
        assert_eq!(individual.fitness, 0.0);
        
        // Check that all characters are in valid ASCII range
        for &ch in &individual.chars {
            assert!(ch >= 0x20 && ch <= 0x7F);
        }
    }
    
    #[test]
    fn test_individual_crossover() {
        let parent1 = Individual::new(vec![b'A'; 10]);
        let parent2 = Individual::new(vec![b'B'; 10]);
        
        let (child1, child2) = parent1.crossover(&parent2, 1.0); // 100% crossover rate
        
        assert_eq!(child1.chars.len(), 10);
        assert_eq!(child2.chars.len(), 10);
        
        // With 100% crossover rate, children should be swapped
        assert_eq!(child1.chars, vec![b'B'; 10]);
        assert_eq!(child2.chars, vec![b'A'; 10]);
    }
    
    #[test]
    fn test_individual_mutation() {
        let mut individual = Individual::new(vec![b'A'; 100]);
        let original = individual.chars.clone();
        
        individual.mutate(1.0); // 100% mutation rate
        
        // With 100% mutation rate, all characters should be different
        assert_ne!(individual.chars, original);
        
        // But they should still be from allowed character set
        for &ch in &individual.chars {
            assert!(ALLOWED_CHARS.contains(&ch));
        }
    }
    
    #[test]
    fn test_genetic_algorithm_creation() {
        let ascii_gen = create_test_ascii_generator();
        let target_img = create_test_target_image();
        
        let ga = GeneticAlgorithm::new(10, 10, 20, &ascii_gen, &target_img, 2, None);
        
        assert_eq!(ga.population.len(), 20);
        assert_eq!(ga.population_size, 20);
        assert_eq!(ga.width, 10);
        assert_eq!(ga.height, 10);
        assert_eq!(ga.thread_count, 2);
        
        // Check that all individuals have correct size
        for individual in &ga.population {
            assert_eq!(individual.chars.len(), 100); // 10 * 10
        }
    }
    
    #[test]
    fn test_fitness_calculation() {
        let ascii_gen = create_test_ascii_generator();
        let target_img = create_test_target_image();
        
        let ga = GeneticAlgorithm::new(2, 2, 10, &ascii_gen, &target_img, 1, None);
        let individual = Individual::new(vec![b' ', b' ', b' ', b' ']); // All spaces
        
        let fitness = ga.calculate_fitness(&individual);
        assert!(fitness >= 0.0 && fitness <= 1.0);
    }
    
    #[test]
    fn test_tournament_selection() {
        let ascii_gen = create_test_ascii_generator();
        let target_img = create_test_target_image();
        
        let mut ga = GeneticAlgorithm::new(2, 2, 10, &ascii_gen, &target_img, 1, None);
        
        // Set different fitness values
        ga.population[0].fitness = 0.9;
        ga.population[1].fitness = 0.1;
        
        let selected = ga.tournament_selection();
        assert!(selected.fitness >= 0.0);
    }
    
    #[test]
    fn test_individual_with_init_char() {
        // Use 'o' which is in our allowed character set
        let individual = Individual::new_with_init_char(100, 'o');
        assert_eq!(individual.chars.len(), 100);
        
        // Count how many characters are 'o' (should be around 95%)
        let o_count = individual.chars.iter().filter(|&&c| c == b'o').count();
        let random_count = individual.chars.iter().filter(|&&c| c != b'o').count();
        
        // Should be approximately 95% 'o' and 5% random (with some variance)
        assert!(o_count >= 90); // At least 90% should be 'o'
        assert!(random_count <= 10); // At most 10% should be random
        assert_eq!(o_count + random_count, 100);
        
        // All random characters should be valid ASCII
        for &c in &individual.chars {
            assert!(c >= 0x20 && c <= 0x7F);
        }
    }
    
    #[test]
    fn test_genetic_algorithm_with_init_char() {
        let ascii_gen = create_test_ascii_generator();
        let target_img = create_test_target_image();
        
        let ga = GeneticAlgorithm::new(3, 3, 5, &ascii_gen, &target_img, 1, Some('#'));
        
        // Check that all individuals in population use the init character
        for individual in &ga.population {
            let hash_count = individual.chars.iter().filter(|&&c| c == b'#').count();
            let total_count = individual.chars.len();
            
            // Should be around 95% '#' characters, but with small sample size (9 chars)
            // we need to allow for statistical variation. Expect at least 70%.
            assert!(hash_count >= (total_count * 70) / 100); // At least 70%
        }
    }
}