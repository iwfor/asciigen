# CLAUDE.md - ASCIIGen Project Documentation

This file provides context and guidance for AI assistants working on the ASCIIGen project.

## Project Overview

ASCIIGen is a Rust application that generates ASCII art from images using either genetic algorithms or brute-force optimization, with parallel processing support. The project was built incrementally with comprehensive testing and modern Rust best practices.

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
   - Population-based evolution with configurable population size (default 80)
   - Tournament selection, uniform crossover, and mutation operations
   - Parallel fitness evaluation using `rayon`
   - Elite preservation (top 10% survive each generation)

5. **`src/brute_force.rs`** - Brute force optimization implementation
   - Position-by-position character optimization for guaranteed optimal results
   - Tests all allowed characters at each grid position
   - Uses same fitness function as genetic algorithm for direct comparison
   - Single-threaded but deterministic approach

### Key Design Decisions

1. **Parallel Processing**: Uses `rayon` for multi-threaded fitness evaluation
   - Significant performance improvements (37-41% faster with 4-8 threads)
   - Thread pool configured based on user input (`-j/--jobs` option)
   - Larger populations (200-800) more effectively utilize high core count systems

2. **Font Rendering**: Uses embedded TrueType font for consistent character rendering
   - Font file must be placed in `assets/DejaVuSansMono.ttf`
   - Characters are pre-rendered and cached for performance

3. **Smart Fitness Function**: Non-background pixel focused evaluation
   - Pre-calculates non-background pixel count from target image
   - Only evaluates meaningful pixels (foreground content)
   - Penalizes false positives (ASCII characters where target is background)
   - Tolerance of 30/255 for pixel intensity differences
   - Returns realistic fitness scores that reflect actual image similarity

4. **Intelligent Initialization**: Background probability-based population creation
   - Calculates percentage of background pixels in target image
   - Uses this probability to place spaces vs characters during initialization
   - Creates realistic starting populations that match target image structure

5. **Background-Aware Mutation**: Maintains realistic character distribution
   - Uses same background probability as initialization during mutation
   - Preserves sparse character placement throughout evolution

6. **Initialization Options**: Support for both random and character-based initialization
   - Random: Uses background probability for realistic distribution
   - Character-based: 95% specified character + 5% random for diversity

7. **Time-Based Progress**: Configurable status update intervals
   - Default 1.0 second intervals with elapsed time tracking
   - Replaces fixed generation-based updates for consistent user feedback

8. **Debug Mode**: Optional debug image output for analysis
   - Saves converted input image as PNG (resized grayscale version)
   - Saves final ASCII art as rendered PNG image (same size as fitness comparison buffer)
   - Files named `debug_input_<filename>.png` and `debug_ascii_<filename>.png`
   - Both images are identical dimensions for pixel-perfect comparison
   - Supports both black and white background modes

9. **Verbose Mode**: Real-time evolution progress display
   - Shows current best ASCII art at each status update interval
   - Helps monitor genetic algorithm convergence
   - Useful for tuning parameters and understanding evolution progress

10. **Limited Character Set**: Optimized character palette for ASCII art
    - Uses curated set: ` <>,./?\\|[]{}-_=+AvViIoOxXwWM`~;:'"!@#$%^&*()8`
    - Provides good visual variety while maintaining readability
    - Avoids problematic characters that don't render well

11. **Background Color Options**: Flexible output formatting
   - Default: White characters on black background (terminal-friendly)
   - White background mode: Black characters on white background (print-friendly)
   - Proper color inversion for cached character images

12. **Dual Optimization Modes**: Genetic algorithm vs brute-force comparison
   - Genetic algorithm: Fast approximation with population-based evolution
   - Brute force: Guaranteed optimal solution with exhaustive character testing
   - Same fitness function for direct performance and quality comparison
   - Compatible with all existing features (UI, debug, verbose modes)

13. **Source Image Inversion**: Color inversion for negative images
   - Inverts all pixel values (255 - pixel_value) before processing
   - Useful for converting negative images or images with inverted brightness
   - Applied during image preprocessing before fitness evaluation
   - Compatible with both genetic algorithm and brute-force modes

## Command Line Interface

```bash
Usage: asciigen [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input image file path

Options:
  -w, --width <WIDTH>              Width in characters
  -H, --height <HEIGHT>            Height in characters
  -g, --generations <GENERATIONS>  Number of generations (0 = continuous mode) [default: 100]
  -j, --jobs <JOBS>                Number of threads [default: 4]
  -p, --population <SIZE>          Population size (20-1000) [default: 80]
  -i, --init-char <INIT_CHAR>      Initialization character (95% + 5% random)
  -o, --output <OUTPUT>            Output file path (optional)
  -d, --debug                      Save debug images (converted input and final ASCII art as PNG files)
  -v, --verbose                    Verbose output: display fittest ASCII art after each progress update
  -W, --white-background           Use white background (default is black background with white characters)
  -s, --status-interval <SECONDS>  Status update interval in seconds [default: 1.0]
      --no-ui                      Disable interactive ncurses UI and use console output instead
  -b, --brute-force                Use brute-force mode instead of genetic algorithm
  -I, --invert-source              Invert source image colors (useful for negative images)
  -h, --help                       Print help
```

### Interactive ncurses UI

By default, ASCIIGen uses an interactive ncurses-based text user interface that provides:

- **Real-time Progress Display**: Shows current generation, progress percentage, and estimated time to completion
- **Fitness Tracking**: Displays current best fitness with color-coded indicators (red < 30%, yellow 30-70%, green > 70%)
- **Performance Metrics**: Accurate real-time generations per second calculation based on total progress, elapsed time, and active thread count
- **Thread Information**: Displays number of threads used for parallel fitness calculation
- **ASCII Dimensions**: Shows target ASCII art size in characters (width × height)
- **Interactive Control**: Press 'q' to quit early, other keys for future controls
- **Live ASCII Preview**: When verbose mode is enabled, shows the current best ASCII art in real-time
- **Visual Progress Bar**: Graphical representation of evolution progress with filled/empty indicators
- **Color-Coded Interface**: Uses terminal colors to highlight important information
- **Automatic Fallback**: Falls back to console output if ncurses initialization fails

The ncurses UI provides a much more engaging and informative experience compared to simple console output.

Use `--no-ui` to disable the interactive interface and use traditional console output instead.

### Continuous Mode

When `--generations 0` is specified, ASCIIGen runs in continuous mode:

- **Infinite Evolution**: Runs until manually stopped (press 'q' in UI or Ctrl+C in console)
- **Fitness-Based Progress**: Progress bar shows current best fitness instead of generation progress
- **No ETA Display**: Time estimates are hidden since there's no target endpoint
- **Real-time Optimization**: Continues improving ASCII art indefinitely
- **Interactive Control**: Essential for stopping the evolution when satisfied with results

Continuous mode is ideal for:
- Finding optimal results without predefined limits
- Interactive experimentation with different fitness levels
- Long-running optimization sessions
- Exploring the maximum potential of the genetic algorithm

### Brute-Force Mode

When `--brute-force` is specified, ASCIIGen uses exhaustive optimization instead of genetic algorithms:

- **Guaranteed Optimal**: Tests all allowed characters at each position to find the absolute best solution
- **Position-by-Position**: Optimizes one character location at a time, building the result incrementally
- **Same Fitness Function**: Uses identical scoring as genetic algorithm for direct comparison
- **Progress Tracking**: Shows completion percentage and current position being optimized
- **Deterministic**: Always produces the same result for the same input and parameters
- **Single-Threaded**: No parallelization, but systematic and thorough approach

Brute-force mode is ideal for:
- Any ASCII art size where optimal results are desired (surprisingly fast performance)
- Benchmarking genetic algorithm performance and quality
- Research and comparison purposes
- When absolute optimality is required
- Baseline "ground truth" generation for algorithm validation

### Validation Rules
- Must specify either width OR height (not both)
- Population size must be between 20 and 1000
- Initialization character must be from the allowed character set
- Thread count should be reasonable (1-16 typically)
- Status interval can be fractional seconds (e.g., 0.5, 2.5)
- For optimal performance, match population size to available CPU cores
- Debug and verbose modes can be used together for comprehensive analysis
- Brute-force mode ignores population size, generations, and thread count parameters
- Brute-force mode has O(positions × characters) complexity but optimized implementation provides excellent real-world performance
- Source image inversion is applied before all other processing and affects both debug output and final results

## Dependencies

### Core Dependencies
- `image = "0.25"` - Image loading and processing
- `fast_image_resize = "4.2"` - High-quality image resizing
- `rusttype = "0.9"` - TrueType font rendering
- `rayon = "1.10"` - Data parallelism
- `clap = "4.5"` - Command-line parsing
- `rand = "0.8"` - Random number generation
- `ncurses = "5.101.0"` - Interactive terminal user interface

### Dev Dependencies
- `mockall = "0.13"` - Mocking for unit tests

## Testing Strategy

### Unit Tests
- **Image Processing**: Test loading, resizing, grayscale conversion, and inversion
- **ASCII Generation**: Test character rendering and caching
- **Genetic Algorithm**: Test individual creation, crossover, mutation, selection
- **Brute Force**: Test character optimization, fitness calculation, and progress tracking
- **Initialization**: Test both random and character-based initialization

### Integration Tests
- Font loading and character rendering
- Complete genetic algorithm runs with small populations
- Complete brute-force runs with small character grids
- CLI argument parsing and validation
- Mode selection and parameter compatibility

### Performance Tests
- Multi-threading performance improvements
- Fitness calculation timing
- Memory usage with different population sizes
- Genetic algorithm vs brute-force comparison benchmarks

## Common Development Tasks

### Adding New CLI Options
1. Add field to `Args` struct in `main.rs`
2. Update `GeneticAlgorithm::evolve()` or other method signatures if needed
3. Pass value through the call chain to relevant functions
4. Update tests that create `GeneticAlgorithm` instances or call affected methods
5. Update help documentation and CLAUDE.md

### Modifying Character Set
- Update `ALLOWED_CHARS` constant in `genetic_algorithm.rs`
- Ensure character cache in `AsciiGenerator` covers all allowed characters
- Update tests to use characters from the new set
- Consider fitness implications of character changes

### Modifying Genetic Algorithm Parameters
- Population size: Configurable via CLI (20-1000), default 80
- Elite size: 10% of population, calculated in `GeneticAlgorithm::new()`
- Mutation rate: 1%, in `GeneticAlgorithm::new()`
- Crossover rate: 80%, in `GeneticAlgorithm::new()`
- Tournament size: 3, in `tournament_selection()`

### Population Size Recommendations
- **1-4 cores**: 40-80 population
- **6-8 cores**: 80-150 population
- **12+ cores**: 200-400 population
- **24+ cores**: 400-800 population

Larger populations provide better genetic diversity but require more CPU cores for efficient parallel processing.

### Adding New Fitness Functions
1. Create new method in `GeneticAlgorithm`
2. Update `calculate_fitness_for_chars_static()` for parallel support
3. Consider creating a trait for different fitness strategies
4. Add comprehensive tests

## Performance Characteristics

### Genetic Algorithm Performance (20x10 characters, 10 generations)
- 1 thread: ~1.1 seconds
- 4 threads: ~0.7 seconds (37% improvement)
- 8 threads: ~0.66 seconds (41% improvement)

### Brute-Force Performance (actual benchmarks)
- 40x25 characters (1000 positions): ~0.7 seconds
- Performance scales approximately linearly with grid size
- Single-threaded but highly optimized character-by-character evaluation

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
5. Limited character set for optimized ASCII art generation
6. Debug mode with image output for analysis and debugging
7. Verbose mode for real-time evolution monitoring
8. Background color options for flexible output formatting
9. Improved character rendering with proper baseline alignment
10. Brute-force optimization mode for guaranteed optimal results
11. Source image color inversion for negative images

Each feature was implemented with full testing and maintains backward compatibility.

### Recent Enhancements

#### Smart Fitness Function (Major Improvement)
- Replaced simple pixel matching with intelligent non-background pixel evaluation
- Pre-calculates non-background pixels in target image for focused scoring
- Penalizes false positives (ASCII characters where target is background)
- Provides realistic fitness scores that reflect actual image similarity
- Eliminates unrealistically high scores from background pixel matching

#### Background Probability-Based Evolution
- Calculates background/foreground ratio from target image during initialization
- Uses this probability for realistic character distribution in initial population
- Applies same probability during mutation to maintain character balance
- Results in dramatically improved convergence and ASCII art quality

#### Time-Based Progress Updates
- Replaced fixed 10-generation intervals with configurable time-based updates
- Default 1.0 second intervals with fractional second support (e.g., 0.5s, 2.5s)
- Shows elapsed time and total runtime for better performance tracking
- Provides consistent user feedback regardless of hardware speed

#### Configurable Population Size
- Added CLI option for population size (20-1000) with default of 80
- Larger populations provide better genetic diversity and solution quality
- Scales effectively with high core count systems (200-800 for 12+ cores)
- Includes validation and performance guidelines for different system configurations

#### Debug Image Improvements
- Debug ASCII images now match fitness comparison buffer dimensions exactly
- Enables pixel-perfect comparison between target and ASCII images
- Fixed character rendering to use proper baseline alignment
- Both debug images are identical size for accurate analysis

#### Character Set Optimization
- Replaced full ASCII range with curated 46-character set
- Improved visual quality and readability of generated ASCII art
- Maintained compatibility with existing initialization and mutation logic

#### Background Color System
- Default: White characters on black background (terminal display)
- Optional: Black characters on white background (printing/documents)
- Proper color inversion maintains character readability in both modes

#### Brute-Force Optimization Mode
- Added deterministic optimization alternative to genetic algorithms
- Position-by-position character optimization for guaranteed optimal results
- Uses identical fitness function as genetic algorithm for direct comparison
- Surprisingly fast performance: 40x25 grid (1000 characters) in ~0.7 seconds
- Full integration with existing UI, debug, and verbose modes
- Ideal for benchmarking genetic algorithm performance and research

#### Source Image Color Inversion
- Added `--invert-source` option for processing negative images
- Inverts all pixel values (255 - original_value) during preprocessing
- Useful for converting film negatives, inverted scans, or stylistic effects
- Compatible with both genetic algorithm and brute-force modes
- Applied before fitness calculation, affecting all processing and debug output