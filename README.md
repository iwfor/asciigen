# ASCIIGen

A Rust application that generates ASCII art from images using genetic algorithms with parallel processing support.  This is a terrible approach to solving this problem but a great demonstration of genetic algorithms.

## Features

- **Genetic Algorithm**: Evolves ASCII art over multiple generations to match source images
- **Parallel Processing**: Multi-threaded fitness evaluation for improved performance
- **Flexible Sizing**: Specify width or height in characters (auto-calculates the other dimension)
- **High-Quality Rendering**: Uses monospace fonts with proper character spacing
- **Configurable Evolution**: Adjustable number of generations and thread count
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
  -o, --output <OUTPUT>            Output file path (optional)
  -h, --help                       Print help
```

### Requirements
- Specify **either** width **or** height (not both)
- Supported image formats: PNG, JPEG, GIF, BMP, TIFF, WebP
- Font file: DejaVu Sans Mono (included in `assets/` directory)

## How It Works

### Genetic Algorithm Process

1. **Initialization**: Creates a population of 40 random ASCII art individuals
2. **Fitness Evaluation**: Compares each ASCII art against the target image pixel-by-pixel
3. **Selection**: Uses tournament selection to choose parents for reproduction
4. **Crossover**: Performs uniform crossover between parent individuals
5. **Mutation**: Randomly mutates characters with a small probability
6. **Elitism**: Preserves the top 10% of individuals across generations

### Technical Implementation

- **Image Processing**: Loads, resizes, and converts images to grayscale
- **Font Rendering**: Renders ASCII characters using TrueType fonts
- **Parallel Fitness**: Uses Rayon for concurrent fitness evaluations
- **Character Set**: Uses 7-bit ASCII characters (0x20-0x7F)
- **Fitness Function**: Pixel matching with tolerance for better convergence

## Performance

The application shows significant performance improvements with multi-threading:

| Threads | Execution Time | Speedup | CPU Usage |
|---------|---------------|---------|-----------|
| 1       | 1.111s        | 1.0x    | 61%       |
| 4       | 0.694s        | 1.6x    | 104%      |
| 8       | 0.658s        | 1.7x    | 119%      |

## Examples

### Input Image
A simple 50x50 pixel image with geometric shapes.

### Generated ASCII Art (20x10 characters)
```
:>-fFu?ER`~O_=6O.UR|
9o#X~<a]/S`P^5x#w.a$
1_B=&2oours}d/+PVT*%
P)IHuV(80$3i3vLRTXA
hWGRTl^h~p)h/^AI$lYi
ZQe 8p9Bhd;vr/xdhu)D
g@v9 Y1C0Ip&30G!C>e
CZVr<B9og,?!5NH\l>ec
jp{S=_E@[x8rksuF@uEL
l3F8aS%e:~QUj6W{KU12
```

### Fitness Evolution
```
Generation 0: Best fitness = 77.00%
Generation 10: Best fitness = 78.50%
Generation 20: Best fitness = 79.20%
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
- **Population Size**: 40 individuals
- **Elite Size**: 10% of population (4 individuals)
- **Mutation Rate**: 1% per character
- **Crossover Rate**: 80%
- **Selection**: Tournament selection (size 3)

### Fitness Function
The fitness function calculates the percentage of pixels that match between the generated ASCII art and the target image:

```rust
fitness = matching_pixels / total_pixels
```

With a tolerance of 30 out of 255 for pixel intensity differences to allow for minor variations.

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