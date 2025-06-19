mod image_processor;
mod ascii_generator;
mod genetic_algorithm;

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
    
    #[arg(short, long, default_value = "100", help = "Number of generations")]
    generations: u32,
    
    #[arg(short, long, default_value = "4", help = "Number of threads for parallel fitness evaluation")]
    jobs: usize,
    
    #[arg(short = 'i', long, help = "Character to initialize art buffers with (95% of characters, 5% random)")]
    init_char: Option<char>,
    
    #[arg(short, long, help = "Output file path (optional)")]
    output: Option<PathBuf>,
    
    #[arg(short = 'd', long, help = "Save debug images (converted input and final ASCII art as PNG files)")]
    debug: bool,
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
    
    println!("Loading image: {:?}", args.input);
    let processor = image_processor::ImageProcessor::new();
    let original_img = processor.load_image(&args.input)?;
    
    let (target_width, target_height) = calculate_dimensions(
        &original_img, 
        args.width, 
        args.height
    );
    
    println!("Target ASCII dimensions: {}x{}", target_width, target_height);
    
    let resized_bw = processor.prepare_target_image(&original_img, target_width, target_height)?;
    
    let ascii_gen = ascii_generator::AsciiGenerator::new();
    
    let mut ga = genetic_algorithm::GeneticAlgorithm::new(
        target_width,
        target_height,
        40, // population size
        &ascii_gen,
        &resized_bw,
        args.jobs,
        args.init_char,
    );
    
    println!("Running genetic algorithm for {} generations...", args.generations);
    let best_individual = ga.evolve(args.generations);
    
    let ascii_art = ascii_gen.individual_to_string(&best_individual, target_width);
    println!("\nBest ASCII art (fitness: {:.2}%):\n{}", best_individual.fitness * 100.0, ascii_art);
    
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
        
        // Save final ASCII art as image
        let ascii_image = ascii_gen.generate_ascii_image(&best_individual.chars, target_width, target_height);
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