# ASCIIGen

A Rust application that generates ASCII art from images using genetic algorithms with parallel processing support.

First of all, this is a terrible approach to solving this particular problem but a great demonstration of genetic
algorithms.  A better approach would be to just brute force each character using the fitness function.  That
would likely be orders of magnitude faster than using the genetic algorithms, but the point of the project is to
complete experiments, not to implement the optimal solution.

This project was mostly generated with Claude Code with Sonnet 4.  Rust was used because the author is learning
Rust.  My biggest contributions were to the final algorithm selected for the fitness function and the random
character distribution of white spaces based on the source image.  I also spent most of my debugging time getting
Claude to correctly size, position, and not clip the ASCII characters when rendering them into the image buffer.

## Author

Isaac Foraker

## Features

- **Genetic Algorithm**: Evolves ASCII art over multiple generations to match source images
- **Parallel Processing**: Multi-threaded fitness evaluation for improved performance
- **Smart Initialization**: Uses background probability to create realistic initial populations
- **Intelligent Fitness**: Non-background pixel focused evaluation with false-positive penalties
- **Flexible Sizing**: Specify width or height in characters (auto-calculates the other dimension)
- **High-Quality Rendering**: Uses monospace fonts with proper character spacing
- **Time-Based Progress**: Configurable status updates at regular time intervals
- **Debug Mode**: Save processed images and ASCII art renderings for analysis
- **Background Options**: Support for both black and white background modes
- **File Output**: Save generated ASCII art to text files

## Installation

### Prerequisites
- Rust (latest stable version)
- Cargo package manager

### Build from Source
```bash
git clone <repository-url>
cd asciigen
cargo build --release
```

## Usage

### Basic Usage
```bash
# Generate ASCII art with 20 character width
cargo run -- image.jpg --width 20

# Generate ASCII art with 15 character height
cargo run -- image.jpg --height 15

# Run for 50 generations with 8 threads
cargo run -- image.jpg --width 30 --generations 50 --jobs 8

# Save output to file
cargo run -- image.jpg --width 25 --output result.txt

# Use character initialization for better convergence
cargo run -- image.jpg --width 25 --init-char 'o'

# Generate debug images with white background
cargo run -- image.jpg --width 15 --debug --white-background

# Verbose mode with real-time progress and custom status interval
cargo run -- image.jpg --width 20 --verbose --generations 100 --status-interval 0.5

# Use larger population for high-core count systems
cargo run -- image.jpg --width 25 --population 200 --jobs 16

# Full featured run with all options
cargo run -- image.jpg --width 25 --generations 50 --jobs 8 --population 120 --init-char '#' --verbose --debug --status-interval 2.0
```

### Command Line Options

```
asciigen [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input image file path

Options:
  -w, --width <WIDTH>              Width in characters
  -H, --height <HEIGHT>            Height in characters
  -g, --generations <GENERATIONS>  Number of generations [default: 100]
  -j, --jobs <JOBS>                Number of threads for parallel fitness evaluation [default: 4]
  -p, --population <SIZE>          Population size (20-1000) [default: 80]
  -i, --init-char <INIT_CHAR>      Character to initialize art buffers with (95% of characters, 5% random)
  -o, --output <OUTPUT>            Output file path (optional)
  -d, --debug                      Save debug images (converted input and final ASCII art as PNG files)
  -v, --verbose                    Verbose output: display fittest ASCII art after each progress update
  -W, --white-background           Use white background (default is black background with white characters)
  -s, --status-interval <SECONDS>  Status update interval in seconds [default: 1.0]
  -h, --help                       Print help
```

### Requirements
- Specify **either** width **or** height (not both)
- Population size must be between 20 and 1000
- Thread count should match your system's capabilities (larger populations benefit from more threads)
- Supported image formats: PNG, JPEG, GIF, BMP, TIFF, WebP
- Font file: DejaVu Sans Mono (included in `assets/` directory)
- Initialization character must be from the allowed character set if specified

### Character Set
ASCIIGen uses an optimized character set designed for ASCII art generation:
```
 <>,./?\\|[]{}-_=+AvViIoOxXwWM`~;:'"!@#$%^&*()8
```

This limited set provides good visual variety while maintaining readability and avoiding problematic characters.

### Debug Mode
When using the `--debug` flag, ASCIIGen saves two PNG files:
- `debug_input_<filename>.png`: The processed input image (resized and grayscale)
- `debug_ascii_<filename>.png`: The final ASCII art rendered as an image (same size as fitness comparison buffer)

Both debug images are the same dimensions, allowing pixel-perfect comparison of what the genetic algorithm is actually optimizing.

## How It Works

### Genetic Algorithm Process

1. **Smart Initialization**: Creates a population of ASCII art individuals (default 80) using background probability
   - Calculates percentage of background pixels in target image
   - Uses this probability to place spaces vs characters during initialization
2. **Intelligent Fitness Evaluation**: Focuses on meaningful pixels rather than background
   - Only evaluates non-background pixels from the target image
   - Awards points for matching pixels within tolerance
   - Penalizes false positives (ASCII characters where target is background)
3. **Selection**: Uses tournament selection to choose parents for reproduction
4. **Crossover**: Performs uniform crossover between parent individuals
5. **Background-Aware Mutation**: Maintains realistic character distribution
   - Uses same background probability as initialization
   - Preserves sparse character placement throughout evolution
6. **Elitism**: Preserves the top 10% of individuals across generations
7. **Time-Based Progress**: Updates status at configurable time intervals

### Technical Implementation

- **Image Processing**: Loads, resizes, and converts images to grayscale with proper dimension matching
- **Font Rendering**: Renders ASCII characters using TrueType fonts with proper baseline alignment
- **Parallel Fitness**: Uses Rayon for concurrent fitness evaluations across multiple threads
- **Character Set**: Uses optimized 46-character set for better ASCII art quality
- **Smart Fitness Function**: Non-background pixel focused evaluation with false-positive penalties
- **Background Probability**: Pre-calculated statistics guide initialization and mutation
- **Time-Based Updates**: Configurable status intervals (default 1.0 seconds) with elapsed time tracking
- **Debug Mode**: Saves processed input and final ASCII art as PNG images for analysis
- **Background Options**: Supports both white-on-black (default) and black-on-white rendering

## Performance

The application shows significant performance improvements with multi-threading and larger populations:

| Threads | Population | Execution Time | Speedup | CPU Usage | Notes |
|---------|------------|---------------|---------|-----------|-------|
| 1       | 80         | 1.111s        | 1.0x    | 61%       | Baseline |
| 4       | 80         | 0.694s        | 1.6x    | 104%      | Good for most systems |
| 8       | 80         | 0.658s        | 1.7x    | 119%      | Diminishing returns |
| 8       | 200        | 1.245s        | 0.9x    | 145%      | Better exploration |
| 16      | 400        | 2.156s        | 0.5x    | 187%      | High-end systems |

**Population Size Guidelines:**
- **Small systems (1-4 cores)**: 40-80 population
- **Mid-range systems (6-8 cores)**: 80-150 population
- **High-end systems (12+ cores)**: 200-400 population
- **Workstations (24+ cores)**: 400-800 population

Larger populations provide better genetic diversity and solution quality but require more cores to be efficient.

## Examples

### Input Image
A simple 50x50 pixel image with geometric shapes.

### Generated ASCII Art (20x10 characters)
```
      |#~X$
    ~8A&} @
       @- o
     @o    =W[
     ([      #{
     @@[     \$
      <[#M=#MAx?(
      M#wM[A&=[
```

### Fitness Evolution with Time-Based Progress
```
Background threshold: 50, Total non-background pixels: 1218, Background probability: 94.2%
Running genetic algorithm for 100 generations with population size 80...
Generation 20: Best fitness = 2.91% (elapsed: 0.5s)
Generation 41: Best fitness = 4.60% (elapsed: 1.0s)
Generation 62: Best fitness = 4.65% (elapsed: 1.5s)
Generation 83: Best fitness = 5.98% (elapsed: 2.0s)
Final generation 99: Best fitness = 7.00% (total time: 2.4s)
```

## Dependencies

- **image**: Image loading and processing
- **fast_image_resize**: High-quality image resizing
- **rusttype**: TrueType font rendering
- **rayon**: Data parallelism for multi-threading
- **clap**: Command-line argument parsing
- **rand**: Random number generation

## Project Structure

```
asciigen/
├── src/
│   ├── main.rs              # CLI interface and main application logic
│   ├── image_processor.rs   # Image loading, resizing, and conversion
│   ├── ascii_generator.rs   # ASCII art generation and font rendering
│   └── genetic_algorithm.rs # Genetic algorithm implementation
├── assets/
│   └── DejaVuSansMono.ttf  # Monospace font for character rendering
├── Cargo.toml              # Project dependencies and metadata
└── README.md               # This file
```

## Development

### Running Tests
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

### Code Coverage
The project includes comprehensive unit tests for:
- Image processing functions
- ASCII art generation
- Genetic algorithm operations
- Individual crossover and mutation

### Building for Release
```bash
cargo build --release
```

## Algorithm Details

### Genetic Algorithm Parameters
- **Population Size**: 80 individuals (configurable 20-1000)
- **Elite Size**: 10% of population (8 individuals with default size)
- **Mutation Rate**: 1% per character
- **Crossover Rate**: 80%
- **Selection**: Tournament selection (size 3)

### Fitness Function
The intelligent fitness function focuses on meaningful pixels rather than background:

```rust
// Only evaluate non-background pixels
for target_lit_pixel in non_background_pixels {
    if ascii_pixel matches target_pixel (within tolerance) {
        score += 1.0
    }
}

// Penalize false positives
for background_pixel in background_pixels {
    if ascii_pixel is lit {
        score -= penalty
    }
}

fitness = score / total_non_background_pixels
```

This approach:
- Only counts pixels that matter (foreground content)
- Penalizes ASCII characters appearing where they shouldn't
- Provides realistic fitness scores that reflect actual image similarity

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is open source. Please check the license file for details.

## Acknowledgments

- Uses the DejaVu Sans Mono font for consistent character rendering
- Built with the Rust ecosystem's excellent crates for image processing and parallel computing
- Inspired by evolutionary art and genetic programming techniques
- Most of this code was generated by Claude Code.
 
### Known issues

- The pause/resume key does not seem to work
