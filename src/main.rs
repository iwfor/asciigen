mod image_processor;
mod ascii_generator;
mod genetic_algorithm;
mod brute_force;
mod ncurses_ui;

use clap::Parser;
use std::path::PathBuf;
use image::GenericImageView;

#[derive(Parser)]
#[command(name = "asciigen")]
#[command(about = "Generate ASCII art from images using genetic algorithms")]
struct Args {
    #[arg(help = "Input image file path")]
    input: PathBuf,

    #[arg(short, long, help = "Width in characters")]
    width: Option<u32>,

    #[arg(short = 'H', long, help = "Height in characters")]
    height: Option<u32>,

    #[arg(short, long, default_value = "100", help = "Number of generations (0 = continuous mode)")]
    generations: u32,

    #[arg(short, long, default_value = "4", help = "Number of threads for parallel fitness evaluation")]
    jobs: usize,

    #[arg(short = 'i', long, help = "Character to initialize art buffers with (95% of characters, 5% random)")]
    init_char: Option<char>,

    #[arg(short, long, help = "Output file path (optional)")]
    output: Option<PathBuf>,

    #[arg(short = 'd', long, help = "Save debug images (converted input and final ASCII art as PNG files)")]
    debug: bool,

    #[arg(short = 'v', long, help = "Verbose output: display fittest ASCII art after each progress update")]
    verbose: bool,

    #[arg(short = 'W', long, help = "Use white background (default is black background with white characters)")]
    white_background: bool,

    #[arg(short = 's', long, default_value = "1.0", help = "Status update interval in seconds")]
    status_interval: f64,

    #[arg(short = 'p', long, default_value = "80", help = "Population size (20-1000)")]
    population: usize,

    #[arg(long, help = "Disable interactive ncurses UI and use console output instead")]
    no_ui: bool,

    #[arg(short = 'b', long, help = "Use brute-force mode instead of genetic algorithm")]
    brute_force: bool,

    #[arg(short = 'I', long, help = "Invert source image colors (useful for negative images)")]
    invert_source: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.width.is_none() && args.height.is_none() {
        eprintln!("Error: Must specify either width or height");
        std::process::exit(1);
    }

    if args.width.is_some() && args.height.is_some() {
        eprintln!("Error: Specify only width OR height, not both");
        std::process::exit(1);
    }

    if args.population < 20 || args.population > 1000 {
        eprintln!("Error: Population size must be between 20 and 1000");
        std::process::exit(1);
    }

    println!("Loading image: {:?}", args.input);
    let processor = image_processor::ImageProcessor::new();
    let original_img = processor.load_image(&args.input)?;

    println!("Input image size: {}x{}", original_img.width(), original_img.height());

    let (target_width, target_height) = calculate_dimensions(
        &original_img,
        args.width,
        args.height
    );

    println!("Target ASCII dimensions: {}x{}", target_width, target_height);

    let ascii_gen = ascii_generator::AsciiGenerator::new();

    // Calculate actual pixel dimensions needed for ASCII character rendering
    let (char_width, char_height) = ascii_gen.char_dimensions();
    let target_pixel_width = target_width * char_width;
    let target_pixel_height = target_height * char_height;

    println!("Character dimensions: {}x{}", char_width, char_height);
    println!("Target pixel dimensions: {}x{}", target_pixel_width, target_pixel_height);

    let resized_bw = processor.prepare_target_image_with_inversion(&original_img, target_pixel_width, target_pixel_height, args.invert_source)?;

    if args.invert_source {
        println!("Source image colors inverted");
    }
    println!("Post-processed input image size: {}x{}", resized_bw.width(), resized_bw.height());

    let (best_individual, total_elapsed) = if args.brute_force {
        // Use brute force mode
        println!("Running brute force generation for {}x{} characters...", target_width, target_height);
        
        let bf_gen = brute_force::BruteForceGenerator::new(
            target_width,
            target_height,
            &ascii_gen,
            &resized_bw,
            args.white_background,
        );

        if args.no_ui {
            // Use console output for brute force
            bf_gen.generate(args.verbose, None::<fn(u32, u32, f64, f64, u32, u32, Option<String>) -> bool>)
        } else {
            // Use ncurses UI for brute force
            match ncurses_ui::NcursesUI::new() {
                Ok(mut ui) => {
                    let result = bf_gen.generate(args.verbose, Some(|position, total_positions, progress, elapsed_time, width, height, ascii_art| {
                        let stats = ncurses_ui::UIStats {
                            generation: position,
                            total_generations: total_positions,
                            best_fitness: progress,
                            elapsed_time,
                            population_size: 1, // Brute force doesn't use population
                            thread_count: 1,    // Brute force is single-threaded
                            width,
                            height,
                            ascii_art,
                        };

                        ui.update(&stats);

                        // Check for user input
                        if let Some(ch) = ui.check_input() {
                            match ch {
                                'q' | 'Q' => return false, // Quit
                                _ => {}
                            }
                        }

                        true // Continue generation
                    }));

                    ui.show_message("Brute force generation complete! Press any key to continue...");
                    ui.check_input(); // Wait for key press
                    result
                },
                Err(e) => {
                    eprintln!("Failed to initialize ncurses UI: {}. Falling back to console output.", e);
                    bf_gen.generate(args.verbose, None::<fn(u32, u32, f64, f64, u32, u32, Option<String>) -> bool>)
                }
            }
        }
    } else {
        // Use genetic algorithm mode
        let mut ga = genetic_algorithm::GeneticAlgorithm::new(
            target_width,
            target_height,
            args.population,
            &ascii_gen,
            &resized_bw,
            args.jobs,
            args.init_char,
            args.white_background,
        );

        if args.generations == 0 {
            println!("Running genetic algorithm in continuous mode with population size {} (press 'q' in UI to stop)...", args.population);
        } else {
            println!("Running genetic algorithm for {} generations with population size {}...", args.generations, args.population);
        }

        if args.no_ui {
            // Use console output
            ga.evolve(args.generations, args.verbose, args.status_interval, None::<fn(u32, u32, f64, f64, usize, usize, u32, u32, Option<String>) -> bool>)
        } else {
            // Use ncurses UI
            match ncurses_ui::NcursesUI::new() {
                Ok(mut ui) => {
                    let result = ga.evolve(args.generations, args.verbose, args.status_interval, Some(|generation, total_generations, best_fitness, elapsed_time, population_size, thread_count, width, height, ascii_art| {
                        let stats = ncurses_ui::UIStats {
                            generation,
                            total_generations,
                            best_fitness,
                            elapsed_time,
                            population_size,
                            thread_count,
                            width,
                            height,
                            ascii_art,
                        };

                        ui.update(&stats);

                        // Check for user input
                        if let Some(ch) = ui.check_input() {
                            match ch {
                                'q' | 'Q' => return false, // Quit
                                _ => {}
                            }
                        }

                        true // Continue evolution
                    }));

                    ui.show_message("Evolution complete! Press any key to continue...");
                    ui.check_input(); // Wait for key press
                    result
                },
                Err(e) => {
                    eprintln!("Failed to initialize ncurses UI: {}. Falling back to console output.", e);
                    ga.evolve(args.generations, args.verbose, args.status_interval, None::<fn(u32, u32, f64, f64, usize, usize, u32, u32, Option<String>) -> bool>)
                }
            }
        }
    };

    // Generate output ASCII image buffer to get its dimensions
    let output_ascii_image = ascii_gen.generate_ascii_image(&best_individual.chars, target_width, target_height);
    println!("Output ASCII image buffer size: {}x{}", output_ascii_image.width(), output_ascii_image.height());

    let ascii_art = ascii_gen.individual_to_string(&best_individual, target_width);
    let mode_str = if args.brute_force { "brute-force" } else { "genetic algorithm" };
    println!("\nBest ASCII art ({}x{} characters, fitness: {:.2}%, mode: {}, elapsed: {:.1}s):\n{}", target_width, target_height, best_individual.fitness * 100.0, mode_str, total_elapsed, ascii_art);

    if let Some(output_path) = args.output {
        std::fs::write(&output_path, ascii_art)?;
        println!("ASCII art saved to: {:?}", output_path);
    }

    // Save debug images if requested
    if args.debug {
        // Save converted input image
        let input_debug_path = format!("debug_input_{}.png",
            args.input.file_stem().unwrap_or_default().to_string_lossy());
        resized_bw.save(&input_debug_path)?;
        println!("Debug input image saved to: {}", input_debug_path);

        // Save final ASCII art as image (same size as fitness comparison buffer)
        let ascii_image = ascii_gen.generate_ascii_image_with_background(&best_individual.chars, target_width, target_height, args.white_background);
        let ascii_debug_path = format!("debug_ascii_{}.png",
            args.input.file_stem().unwrap_or_default().to_string_lossy());
        ascii_image.save(&ascii_debug_path)?;
        println!("Debug ASCII image saved to: {}", ascii_debug_path);
    }

    Ok(())
}

fn calculate_dimensions(
    img: &image::DynamicImage,
    width: Option<u32>,
    height: Option<u32>
) -> (u32, u32) {
    let (img_width, img_height) = img.dimensions();
    let aspect_ratio = img_width as f32 / img_height as f32;

    match (width, height) {
        (Some(w), None) => {
            let h = (w as f32 / aspect_ratio * 0.5) as u32; // Account for character aspect ratio
            (w, h.max(1))
        },
        (None, Some(h)) => {
            let w = (h as f32 * aspect_ratio * 2.0) as u32; // Account for character aspect ratio
            (w.max(1), h)
        },
        _ => unreachable!(), // Already validated in main
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbImage};

    #[test]
    fn test_calculate_dimensions_from_width() {
        let img = DynamicImage::ImageRgb8(RgbImage::new(100, 50));
        let (w, h) = calculate_dimensions(&img, Some(80), None);
        assert_eq!(w, 80);
        assert!(h > 0);
        assert!(h < 80); // Should be less due to aspect ratio adjustment
    }

    #[test]
    fn test_calculate_dimensions_from_height() {
        let img = DynamicImage::ImageRgb8(RgbImage::new(100, 50));
        let (w, h) = calculate_dimensions(&img, None, Some(40));
        assert_eq!(h, 40);
        assert!(w > 40); // Should be more due to aspect ratio
    }
}
