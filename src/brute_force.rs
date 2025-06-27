use crate::ascii_generator::AsciiGenerator;
use crate::genetic_algorithm::{Individual, ALLOWED_CHARS};
use image::{ImageBuffer, Luma};

/// Brute force ASCII art generator that finds the best character for each position
pub struct BruteForceGenerator<'a> {
    width: u32,
    height: u32,
    ascii_generator: &'a AsciiGenerator,
    target_image: &'a ImageBuffer<Luma<u8>, Vec<u8>>,
    total_non_background_pixels: f64,
    background_threshold: u8,
}

impl<'a> BruteForceGenerator<'a> {
    /// Creates a new brute force generator instance
    pub fn new(
        width: u32,
        height: u32,
        ascii_generator: &'a AsciiGenerator,
        target_image: &'a ImageBuffer<Luma<u8>, Vec<u8>>,
        white_background: bool,
    ) -> Self {
        // Calculate background threshold and count non-background pixels
        let background_threshold = if white_background { 200 } else { 50 };
        let total_non_background_pixels = Self::count_non_background_pixels(target_image, background_threshold, white_background);

        println!("Brute force - Background threshold: {}, Total non-background pixels: {}",
                 background_threshold, total_non_background_pixels);

        Self {
            width,
            height,
            ascii_generator,
            target_image,
            total_non_background_pixels,
            background_threshold,
        }
    }

    /// Counts pixels that are not background color in the target image
    fn count_non_background_pixels(
        target_image: &ImageBuffer<Luma<u8>, Vec<u8>>,
        background_threshold: u8,
        white_background: bool,
    ) -> f64 {
        let mut count = 0;

        for pixel in target_image.pixels() {
            let intensity = pixel[0];

            let is_non_background = if white_background {
                intensity < background_threshold
            } else {
                intensity > background_threshold
            };

            if is_non_background {
                count += 1;
            }
        }

        count as f64
    }

    /// Generates ASCII art using brute force approach with optional callback for progress
    pub fn generate<F>(&self, verbose: bool, mut progress_callback: Option<F>) -> (Individual, f64)
    where
        F: FnMut(u32, u32, f64, f64, u32, u32, Option<String>) -> bool,
    {
        use std::time::Instant;

        let start_time = Instant::now();
        let total_positions = (self.width * self.height) as u32;
        let mut best_chars = vec![b' '; total_positions as usize];

        println!("Starting brute force generation for {} positions...", total_positions);

        // Process each character position
        for position in 0..total_positions {
            let row = position / self.width;
            let col = position % self.width;

            // Find the best character for this position
            let best_char = self.find_best_char_for_position(row, col, &best_chars, position as usize);
            best_chars[position as usize] = best_char;

            // Update progress
            if let Some(ref mut callback) = progress_callback {
                let progress = (position + 1) as f64 / total_positions as f64;
                let elapsed = start_time.elapsed().as_secs_f64();
                
                let ascii_art = if verbose {
                    Some(self.ascii_generator.individual_to_string(&Individual::new(best_chars.clone()), self.width))
                } else {
                    None
                };

                let should_continue = callback(
                    position + 1,
                    total_positions,
                    progress,
                    elapsed,
                    self.width,
                    self.height,
                    ascii_art
                );

                if !should_continue {
                    println!("Brute force generation stopped by user");
                    break;
                }
            } else if (position + 1) % 10 == 0 || position + 1 == total_positions {
                let progress = (position + 1) as f64 / total_positions as f64;
                let elapsed = start_time.elapsed().as_secs_f64();
                println!("Progress: {}/{} positions ({:.1}%) - elapsed: {:.1}s", 
                         position + 1, total_positions, progress * 100.0, elapsed);
            }
        }

        let total_elapsed = start_time.elapsed().as_secs_f64();
        let final_individual = Individual::new(best_chars);
        
        // Calculate final fitness
        let final_fitness = self.calculate_fitness(&final_individual);
        let mut result = final_individual;
        result.fitness = final_fitness;

        println!("Brute force generation complete! Final fitness: {:.2}% (total time: {:.1}s)",
                 final_fitness * 100.0, total_elapsed);

        (result, total_elapsed)
    }

    /// Finds the best character for a specific position by testing all allowed characters
    fn find_best_char_for_position(&self, row: u32, col: u32, current_chars: &[u8], position: usize) -> u8 {
        let mut best_char = b' ';
        let mut best_fitness = 0.0;

        // Test each allowed character at this position
        for &test_char in ALLOWED_CHARS {
            let mut test_chars = current_chars.to_vec();
            test_chars[position] = test_char;

            // Calculate fitness for this character choice
            let fitness = self.calculate_fitness_for_position(row, col, test_char);
            
            if fitness > best_fitness {
                best_fitness = fitness;
                best_char = test_char;
            }
        }

        best_char
    }

    /// Calculates fitness for a specific character at a specific position
    fn calculate_fitness_for_position(&self, row: u32, col: u32, test_char: u8) -> f64 {
        // Create a single-character ASCII art image for this position
        let single_char_chars = vec![test_char];
        let single_char_image = self.ascii_generator.generate_ascii_image(&single_char_chars, 1, 1);
        
        // Get character dimensions
        let (char_width, char_height) = self.ascii_generator.char_dimensions();
        
        // Calculate the pixel region in the target image that corresponds to this character position
        let start_x = col * char_width;
        let start_y = row * char_height;
        let end_x = (start_x + char_width).min(self.target_image.width());
        let end_y = (start_y + char_height).min(self.target_image.height());
        
        let mut score = 0.0;
        let mut total_relevant_pixels = 0.0;
        
        // Compare pixels in the character's region
        for y in start_y..end_y {
            for x in start_x..end_x {
                let target_pixel = self.target_image.get_pixel(x, y)[0];
                let target_is_lit = target_pixel > self.background_threshold;
                
                // Get corresponding pixel from the single character image
                let char_x = x - start_x;
                let char_y = y - start_y;
                
                if char_x < single_char_image.width() && char_y < single_char_image.height() {
                    let ascii_pixel = single_char_image.get_pixel(char_x, char_y)[0];
                    let ascii_is_lit = ascii_pixel > self.background_threshold;
                    
                    // Only score meaningful pixels (target non-background)
                    if target_is_lit {
                        total_relevant_pixels += 1.0;
                        let diff = (ascii_pixel as i32 - target_pixel as i32).abs();
                        
                        if diff < 30 { // Same tolerance as genetic algorithm
                            score += 1.0;
                        }
                    } else if ascii_is_lit {
                        // Small penalty for false positives
                        score -= 0.005;
                    }
                }
            }
        }
        
        // Return fitness for this character position
        if total_relevant_pixels > 0.0 {
            let fitness = score / total_relevant_pixels;
            if fitness < 0.0 { 0.0 } else { fitness }
        } else {
            // If no relevant pixels, prefer space character
            if test_char == b' ' { 1.0 } else { 0.0 }
        }
    }

    /// Calculates overall fitness using the same method as genetic algorithm
    fn calculate_fitness(&self, individual: &Individual) -> f64 {
        let ascii_image = self.ascii_generator.generate_ascii_image(&individual.chars, self.width, self.height);

        if self.total_non_background_pixels == 0.0 {
            return 0.0;
        }

        let min_width = ascii_image.width().min(self.target_image.width());
        let min_height = ascii_image.height().min(self.target_image.height());

        let mut score = 0.0;

        for y in 0..min_height {
            for x in 0..min_width {
                let ascii_pixel = ascii_image.get_pixel(x, y)[0];
                let target_pixel = self.target_image.get_pixel(x, y)[0];

                let ascii_is_lit = ascii_pixel > self.background_threshold;
                let target_is_lit = target_pixel > self.background_threshold;

                if target_is_lit {
                    let diff = (ascii_pixel as i32 - target_pixel as i32).abs();
                    if diff < 30 {
                        score += 1.0;
                    }
                } else if ascii_is_lit {
                    score -= 0.005;
                }
            }
        }

        let fitness = score / self.total_non_background_pixels;
        if fitness < 0.0 { 0.0 } else { fitness }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ascii_generator::AsciiGenerator;
    use image::ImageBuffer;

    fn create_test_ascii_generator() -> AsciiGenerator {
        AsciiGenerator::new()
    }

    fn create_test_target_image() -> ImageBuffer<Luma<u8>, Vec<u8>> {
        ImageBuffer::new(20, 20)
    }

    #[test]
    fn test_brute_force_generator_creation() {
        let ascii_gen = create_test_ascii_generator();
        let target_img = create_test_target_image();

        let bf_gen = BruteForceGenerator::new(2, 2, &ascii_gen, &target_img, false);

        assert_eq!(bf_gen.width, 2);
        assert_eq!(bf_gen.height, 2);
        assert_eq!(bf_gen.background_threshold, 50);
    }

    #[test]
    fn test_find_best_char_for_position() {
        let ascii_gen = create_test_ascii_generator();
        let target_img = create_test_target_image();
        let bf_gen = BruteForceGenerator::new(2, 2, &ascii_gen, &target_img, false);

        let current_chars = vec![b' '; 4];
        let best_char = bf_gen.find_best_char_for_position(0, 0, &current_chars, 0);

        // Should return a valid character from the allowed set
        assert!(ALLOWED_CHARS.contains(&best_char));
    }

    #[test]
    fn test_fitness_calculation() {
        let ascii_gen = create_test_ascii_generator();
        let target_img = create_test_target_image();
        let bf_gen = BruteForceGenerator::new(2, 2, &ascii_gen, &target_img, false);

        let individual = Individual::new(vec![b' ', b' ', b' ', b' ']);
        let fitness = bf_gen.calculate_fitness(&individual);

        assert!(fitness >= 0.0 && fitness <= 1.0);
    }
}