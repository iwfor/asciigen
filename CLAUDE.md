# CLAUDE.md - ASCIIGen Project Documentation

This file provides context and guidance for AI assistants working on the ASCIIGen project.

## Project Overview

ASCIIGen is a Rust application that generates ASCII art from images using genetic algorithms with parallel processing support. The project was built incrementally with comprehensive testing and modern Rust best practices.

## Architecture

### Core Modules

1. **`src/main.rs`** - CLI interface and application entry point
   - Uses `clap` for command-line argument parsing
   - Coordinates all modules to run the genetic algorithm
   - Handles user input validation and output

2. **`src/image_processor.rs`** - Image loading and processing
   - Loads images using the `image` crate
   - Resizes images with high-quality Lanczos3 filtering via `fast_image_resize`
   - Converts images to grayscale for fitness comparison
   - All methods include comprehensive documentation and error handling

3. **`src/ascii_generator.rs`** - ASCII art generation and font rendering
   - Renders ASCII characters using TrueType fonts via `rusttype`
   - Caches all 7-bit ASCII characters (0x20-0x7F) for performance
   - Generates ASCII art images from character arrays
   - Uses DejaVu Sans Mono font (included in `assets/` directory)

4. **`src/genetic_algorithm.rs`** - Genetic algorithm implementation
   - Population-based evolution with 40 individuals
   - Tournament selection, uniform crossover, and mutation operations
   - Parallel fitness evaluation using `rayon`
   - Elite preservation (top 10% survive each generation)

### Key Design Decisions

1. **Parallel Processing**: Uses `rayon` for multi-threaded fitness evaluation
   - Significant performance improvements (37-41% faster with 4-8 threads)
   - Thread pool configured based on user input (`-j/--jobs` option)

2. **Font Rendering**: Uses embedded TrueType font for consistent character rendering
   - Font file must be placed in `assets/DejaVuSansMono.ttf`
   - Characters are pre-rendered and cached for performance

3. **Fitness Function**: Pixel-by-pixel comparison with tolerance
   - Tolerance of 30/255 for pixel intensity differences
   - Returns percentage match (0.0 to 1.0)

4. **Initialization Options**: Support for both random and character-based initialization
   - Random: Completely random ASCII characters
   - Character-based: 95% specified character + 5% random for diversity

5. **Debug Mode**: Optional debug image output for analysis
   - Saves converted input image as PNG (resized grayscale version)
   - Saves final ASCII art as rendered PNG image
   - Files named `debug_input_<filename>.png` and `debug_ascii_<filename>.png`

## Command Line Interface

```bash
Usage: asciigen [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input image file path

Options:
  -w, --width <WIDTH>              Width in characters
  -H, --height <HEIGHT>            Height in characters
  -g, --generations <GENERATIONS>  Number of generations [default: 100]
  -j, --jobs <JOBS>                Number of threads [default: 4]
  -i, --init-char <INIT_CHAR>      Initialization character (95% + 5% random)
  -o, --output <OUTPUT>            Output file path (optional)
  -d, --debug                      Save debug images (converted input and final ASCII art as PNG files)
  -h, --help                       Print help
```

### Validation Rules
- Must specify either width OR height (not both)
- Initialization character must be valid ASCII (0x20-0x7F)
- Thread count should be reasonable (1-16 typically)

## Dependencies

### Core Dependencies
- `image = "0.25"` - Image loading and processing
- `fast_image_resize = "4.2"` - High-quality image resizing
- `rusttype = "0.9"` - TrueType font rendering
- `rayon = "1.10"` - Data parallelism
- `clap = "4.5"` - Command-line parsing
- `rand = "0.8"` - Random number generation

### Dev Dependencies
- `mockall = "0.13"` - Mocking for unit tests

## Testing Strategy

### Unit Tests
- **Image Processing**: Test loading, resizing, and grayscale conversion
- **ASCII Generation**: Test character rendering and caching
- **Genetic Algorithm**: Test individual creation, crossover, mutation, selection
- **Initialization**: Test both random and character-based initialization

### Integration Tests
- Font loading and character rendering
- Complete genetic algorithm runs with small populations
- CLI argument parsing and validation

### Performance Tests
- Multi-threading performance improvements
- Fitness calculation timing
- Memory usage with different population sizes

## Common Development Tasks

### Adding New CLI Options
1. Add field to `Args` struct in `main.rs`
2. Update `GeneticAlgorithm::new()` signature if needed
3. Pass value through the call chain
4. Update tests that create `GeneticAlgorithm` instances
5. Update help documentation

### Modifying Genetic Algorithm Parameters
- Population size: Currently 40, defined in `main.rs`
- Elite size: 10% of population, in `GeneticAlgorithm::new()`
- Mutation rate: 1%, in `GeneticAlgorithm::new()`
- Crossover rate: 80%, in `GeneticAlgorithm::new()`
- Tournament size: 3, in `tournament_selection()`

### Adding New Fitness Functions
1. Create new method in `GeneticAlgorithm`
2. Update `calculate_fitness_for_chars_static()` for parallel support
3. Consider creating a trait for different fitness strategies
4. Add comprehensive tests

## Performance Characteristics

### Typical Performance (20x10 characters, 10 generations)
- 1 thread: ~1.1 seconds
- 4 threads: ~0.7 seconds (37% improvement)
- 8 threads: ~0.66 seconds (41% improvement)

### Memory Usage
- Font caching: ~95 character images in memory
- Population: 40 individuals × character array size
- Target image: Full resolution + resized grayscale copy

## Troubleshooting

### Common Issues

1. **Font Loading Failures**
   - Ensure `assets/DejaVuSansMono.ttf` exists and is a valid TTF file
   - Check file permissions
   - Font is embedded at compile time via `include_bytes!`

2. **Compilation Errors with Rayon**
   - Ensure thread pool is initialized before use
   - Use `Arc` for thread-safe sharing of references
   - Be careful with borrowing in parallel contexts

3. **Poor Fitness Convergence**
   - Try different initialization characters
   - Increase population size or generations
   - Check image preprocessing (size, contrast)

4. **Performance Issues**
   - Profile fitness calculation bottlenecks
   - Consider image size vs. character count ratio
   - Monitor thread utilization

### Debugging Tips

1. **Enable Debug Logging**
   ```rust
   println!("Generation {}: Best fitness = {:.2}%", generation, best_fitness * 100.0);
   ```

2. **Visualize Population Diversity**
   - Print character histograms
   - Track fitness distribution across population

3. **Profile Performance**
   ```bash
   cargo build --release
   time ./target/release/asciigen image.jpg --width 20
   ```

## Code Style and Conventions

### Rust Conventions
- Use `cargo fmt` for consistent formatting
- Follow Rust naming conventions (snake_case, etc.)
- Comprehensive documentation with `///` comments
- Error handling with `Result<T, E>` types

### Testing Conventions
- Unit tests in same file as implementation
- Integration tests in `tests/` directory (if added)
- Use descriptive test names: `test_function_name_condition`
- Mock external dependencies where appropriate

### Documentation Standards
- Document all public APIs
- Include usage examples in doc comments
- Explain algorithm parameters and their effects
- Document performance characteristics

## Future Enhancement Ideas

### Algorithm Improvements
- Adaptive mutation rates based on fitness stagnation
- Multiple fitness functions (edge detection, contrast, etc.)
- Crossover strategies beyond uniform crossover
- Population diversity metrics and maintenance

### Feature Additions
- Support for colored ASCII art
- Interactive mode with real-time evolution display
- Batch processing of multiple images
- Custom character sets beyond ASCII
- Image preprocessing options (contrast, brightness)

### Performance Optimizations
- SIMD instructions for fitness calculation
- GPU acceleration for parallel processing
- Memory pool for character image caching
- Incremental fitness calculation

### User Experience
- Progress bars for long runs
- Web interface for easy usage
- Configuration files for complex setups
- Better error messages and validation

## Version History Notes

The project was built incrementally with these major milestones:
1. Basic genetic algorithm with single-threaded fitness
2. Multi-threaded parallel fitness evaluation
3. Initialization character support with 95%/5% split
4. Comprehensive testing and documentation

Each feature was implemented with full testing and maintains backward compatibility.