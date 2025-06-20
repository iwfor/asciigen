use ncurses::*;
use std::time::Instant;

/// Interactive ncurses UI for displaying genetic algorithm progress
pub struct NcursesUI {
    start_time: Instant,
    last_generation: u32,
    last_update_time: Instant,
}

/// Statistics to display in the UI
pub struct UIStats {
    pub generation: u32,
    pub total_generations: u32,
    pub best_fitness: f64,
    pub elapsed_time: f64,
    pub population_size: usize,
    pub thread_count: usize,
    pub width: u32,
    pub height: u32,
    pub ascii_art: Option<String>,
}

impl NcursesUI {
    /// Initialize ncurses and create a new UI instance
    pub fn new() -> Result<Self, String> {
        // Initialize ncurses
        if initscr() == std::ptr::null_mut() {
            return Err("Failed to initialize ncurses".to_string());
        }

        // Set up ncurses options
        cbreak();           // Disable line buffering
        noecho();           // Don't echo keys to screen
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE); // Hide cursor
        timeout(0);         // Non-blocking input

        // Check if terminal supports colors
        if has_colors() {
            start_color();
            // Define color pairs
            init_pair(1, COLOR_GREEN, COLOR_BLACK);  // Success/good values
            init_pair(2, COLOR_YELLOW, COLOR_BLACK); // Warnings/medium values
            init_pair(3, COLOR_RED, COLOR_BLACK);    // Errors/bad values
            init_pair(4, COLOR_CYAN, COLOR_BLACK);   // Headers/titles
            init_pair(5, COLOR_WHITE, COLOR_BLACK);  // Normal text
        }

        // Clear screen
        clear();
        refresh();

        Ok(Self {
            start_time: Instant::now(),
            last_generation: 0,
            last_update_time: Instant::now(),
        })
    }

    /// Update the display with current statistics
    pub fn update(&mut self, stats: &UIStats) {
        // Update timing information
        self.last_generation = stats.generation;
        self.last_update_time = Instant::now();

        // Clear screen and reset cursor
        clear();
        mv(0, 0);

        // Draw header
        self.draw_header();

        // Draw main statistics
        self.draw_stats(stats);

        // Draw progress bar
        if stats.total_generations == 0 {
            self.draw_fitness_progress_bar(stats.best_fitness);
        } else {
            self.draw_progress_bar(stats.generation, stats.total_generations);
        }

        // Draw ASCII art if provided
        if let Some(ref art) = stats.ascii_art {
            self.draw_ascii_art(art);
        }

        // Draw footer with controls
        self.draw_footer();

        // Refresh screen
        refresh();
    }

    /// Draw the header section
    fn draw_header(&self) {
        attron(COLOR_PAIR(4)); // Cyan for header
        mvprintw(0, 0, "ASCIIGen - Genetic Algorithm ASCII Art Generator");
        mvprintw(1, 0, "================================================");
        attroff(COLOR_PAIR(4));
    }

    /// Draw the main statistics section
    fn draw_stats(&self, stats: &UIStats) {
        let y_start = 3;
        let continuous_mode = stats.total_generations == 0;

        // Generation info
        attron(COLOR_PAIR(5)); // White for labels
        mvprintw(y_start, 0, "Generation:");
        attroff(COLOR_PAIR(5));
        attron(COLOR_PAIR(1)); // Green for values
        if continuous_mode {
            mvprintw(y_start, 15, &format!("{} (continuous)", stats.generation));
        } else {
            mvprintw(y_start, 15, &format!("{}/{}", stats.generation, stats.total_generations));
        }
        attroff(COLOR_PAIR(1));

        // Progress percentage (fitness-based in continuous mode, generation-based otherwise)
        let progress = if continuous_mode {
            stats.best_fitness * 100.0 // Fitness as percentage
        } else {
            (stats.generation as f64 / stats.total_generations as f64) * 100.0
        };

        attron(COLOR_PAIR(5));
        if continuous_mode {
            mvprintw(y_start, 35, "Fitness:");
        } else {
            mvprintw(y_start, 35, "Progress:");
        }
        attroff(COLOR_PAIR(5));
        let color = if progress < 25.0 { 3 } else if progress < 75.0 { 2 } else { 1 };
        attron(COLOR_PAIR(color));
        mvprintw(y_start, 45, &format!("{:.1}%", progress));
        attroff(COLOR_PAIR(color));

        // Best fitness
        attron(COLOR_PAIR(5));
        mvprintw(y_start + 1, 0, "Best Fitness:");
        attroff(COLOR_PAIR(5));
        let fitness_color = if stats.best_fitness < 0.3 { 3 } else if stats.best_fitness < 0.7 { 2 } else { 1 };
        attron(COLOR_PAIR(fitness_color));
        mvprintw(y_start + 1, 15, &format!("{:.2}%", stats.best_fitness * 100.0));
        attroff(COLOR_PAIR(fitness_color));

        // Population size
        attron(COLOR_PAIR(5));
        mvprintw(y_start + 1, 35, "Population:");
        attroff(COLOR_PAIR(5));
        attron(COLOR_PAIR(1));
        mvprintw(y_start + 1, 47, &format!("{}", stats.population_size));
        attroff(COLOR_PAIR(1));

        // Elapsed time
        attron(COLOR_PAIR(5));
        mvprintw(y_start + 2, 0, "Elapsed Time:");
        attroff(COLOR_PAIR(5));
        attron(COLOR_PAIR(1));
        mvprintw(y_start + 2, 15, &format!("{:.1}s", stats.elapsed_time));
        attroff(COLOR_PAIR(1));

        // Thread count
        attron(COLOR_PAIR(5));
        mvprintw(y_start + 2, 35, "Threads:");
        attroff(COLOR_PAIR(5));
        attron(COLOR_PAIR(1));
        mvprintw(y_start + 2, 44, &format!("{}", stats.thread_count));
        attroff(COLOR_PAIR(1));

        // Generations per second
        let gens_per_sec = self.calculate_generations_per_second(stats.generation);
        attron(COLOR_PAIR(5));
        mvprintw(y_start + 2, 55, "Gen/s:");
        attroff(COLOR_PAIR(5));
        attron(COLOR_PAIR(1));
        mvprintw(y_start + 2, 62, &format!("{:.2}", gens_per_sec));
        attroff(COLOR_PAIR(1));

        // ASCII Art Dimensions
        attron(COLOR_PAIR(5));
        mvprintw(y_start + 3, 0, "ASCII Size:");
        attroff(COLOR_PAIR(5));
        attron(COLOR_PAIR(1));
        mvprintw(y_start + 3, 15, &format!("{}x{} chars", stats.width, stats.height));
        attroff(COLOR_PAIR(1));

        // ETA (Estimated Time of Arrival) - only show in non-continuous mode
        if !continuous_mode && stats.generation > 0 && gens_per_sec > 0.0 {
            let remaining_gens = stats.total_generations - stats.generation;
            let eta_seconds = remaining_gens as f64 / gens_per_sec;
            attron(COLOR_PAIR(5));
            mvprintw(y_start + 3, 35, "ETA:");
            attroff(COLOR_PAIR(5));
            attron(COLOR_PAIR(2));
            mvprintw(y_start + 3, 40, &format!("{:.1}s", eta_seconds));
            attroff(COLOR_PAIR(2));
        } else if continuous_mode {
            // In continuous mode, show a message instead of ETA
            attron(COLOR_PAIR(4));
            mvprintw(y_start + 3, 35, "Press 'q' to stop");
            attroff(COLOR_PAIR(4));
        }
    }

    /// Draw a progress bar
    fn draw_progress_bar(&self, current: u32, total: u32) {
        let y = 9;
        let bar_width = 60;
        let progress = current as f64 / total as f64;
        let filled = (bar_width as f64 * progress) as usize;

        attron(COLOR_PAIR(5));
        mvprintw(y, 0, "Progress: [");
        attroff(COLOR_PAIR(5));

        // Draw filled portion
        attron(COLOR_PAIR(1));
        for i in 0..filled {
            mvaddch(y, 11 + i as i32, '#' as u32);
        }
        attroff(COLOR_PAIR(1));

        // Draw empty portion
        attron(COLOR_PAIR(5));
        for i in filled..bar_width {
            mvaddch(y, 11 + i as i32, '-' as u32);
        }
        mvaddch(y, 11 + bar_width as i32, ']' as u32);
        attroff(COLOR_PAIR(5));
    }

    /// Draw a fitness-based progress bar for continuous mode
    fn draw_fitness_progress_bar(&self, fitness: f64) {
        let y = 9;
        let bar_width = 60;
        let progress = fitness; // fitness is already 0.0 to 1.0
        let filled = (bar_width as f64 * progress) as usize;

        attron(COLOR_PAIR(5));
        mvprintw(y, 0, "Fitness:  [");
        attroff(COLOR_PAIR(5));

        // Draw filled portion with color based on fitness level
        let color = if fitness < 0.3 { 3 } else if fitness < 0.7 { 2 } else { 1 };
        attron(COLOR_PAIR(color));
        for i in 0..filled {
            mvaddch(y, 11 + i as i32, '=' as u32);
        }
        attroff(COLOR_PAIR(color));

        // Draw empty portion
        attron(COLOR_PAIR(5));
        for i in filled..bar_width {
            mvaddch(y, 11 + i as i32, '.' as u32);
        }
        mvaddch(y, 11 + bar_width as i32, ']' as u32);
        attroff(COLOR_PAIR(5));
    }

    /// Draw ASCII art if provided
    fn draw_ascii_art(&self, art: &str) {
        let y_start = 11;
        let mut max_y = 0;
        let mut max_x = 0;
        getmaxyx(stdscr(), &mut max_y, &mut max_x);

        attron(COLOR_PAIR(4));
        mvprintw(y_start, 0, "Current Best ASCII Art:");
        attroff(COLOR_PAIR(4));

        attron(COLOR_PAIR(5));
        for (i, line) in art.lines().enumerate() {
            let y_pos = y_start + 2 + i as i32;
            // Only draw if we have space and don't overlap with footer
            if y_pos < max_y - 3 {
                // Truncate line if it's too long for the screen
                let display_line = if line.len() > (max_x - 1) as usize {
                    &line[..(max_x - 1) as usize]
                } else {
                    line
                };
                mv(y_pos, 0);
                addstr(display_line);
            }
        }
        attroff(COLOR_PAIR(5));
    }

    /// Draw footer with control information
    fn draw_footer(&self) {
        let mut max_y = 0;
        let mut max_x = 0;
        getmaxyx(stdscr(), &mut max_y, &mut max_x);

        attron(COLOR_PAIR(4));
        mvprintw(max_y - 2, 0, "Controls: 'q' to quit, 'p' to pause/resume");
        mvprintw(max_y - 1, 0, "Press any key to continue...");
        attroff(COLOR_PAIR(4));
    }

    /// Calculate generations per second based on overall progress
    fn calculate_generations_per_second(&self, current_generation: u32) -> f64 {
        if current_generation == 0 {
            return 0.0;
        }

        let elapsed = self.last_update_time.duration_since(self.start_time).as_secs_f64();
        if elapsed > 0.0 {
            current_generation as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Check for user input (non-blocking)
    pub fn check_input(&self) -> Option<char> {
        let ch = getch();
        if ch == ERR {
            None
        } else {
            Some(ch as u8 as char)
        }
    }

    /// Display a message and wait for user input
    pub fn show_message(&self, message: &str) {
        let mut max_y = 0;
        let mut max_x = 0;
        getmaxyx(stdscr(), &mut max_y, &mut max_x);

        attron(COLOR_PAIR(2));
        mvprintw(max_y - 3, 0, message);
        attroff(COLOR_PAIR(2));
        refresh();
    }

    /// Clean up ncurses
    pub fn cleanup(&self) {
        endwin();
    }
}

impl Drop for NcursesUI {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_ui() -> NcursesUI {
        // Create UI without initializing ncurses for testing
        NcursesUI {
            start_time: Instant::now(),
            last_generation: 0,
            last_update_time: Instant::now(),
        }
    }

    #[test]
    fn test_calculate_generations_per_second_zero_generations() {
        let ui = create_test_ui();
        let result = ui.calculate_generations_per_second(0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_calculate_generations_per_second_normal_case() {
        let mut ui = create_test_ui();

        // Simulate 2 seconds elapsed
        ui.last_update_time = ui.start_time + Duration::from_secs(2);

        // Test 10 generations in 2 seconds = 5.0 Gen/s
        let result = ui.calculate_generations_per_second(10);
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_calculate_generations_per_second_fractional_time() {
        let mut ui = create_test_ui();

        // Simulate 0.5 seconds elapsed
        ui.last_update_time = ui.start_time + Duration::from_millis(500);

        // Test 3 generations in 0.5 seconds = 6.0 Gen/s
        let result = ui.calculate_generations_per_second(3);
        assert_eq!(result, 6.0);
    }

    #[test]
    fn test_calculate_generations_per_second_one_generation() {
        let mut ui = create_test_ui();

        // Simulate 1 second elapsed
        ui.last_update_time = ui.start_time + Duration::from_secs(1);

        // Test 1 generation in 1 second = 1.0 Gen/s
        let result = ui.calculate_generations_per_second(1);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_calculate_generations_per_second_high_rate() {
        let mut ui = create_test_ui();

        // Simulate 100ms elapsed
        ui.last_update_time = ui.start_time + Duration::from_millis(100);

        // Test 2 generations in 0.1 seconds = 20.0 Gen/s
        let result = ui.calculate_generations_per_second(2);
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_calculate_generations_per_second_very_small_time() {
        let mut ui = create_test_ui();

        // Simulate 1ms elapsed
        ui.last_update_time = ui.start_time + Duration::from_millis(1);

        // Test 1 generation in 0.001 seconds = 1000.0 Gen/s
        let result = ui.calculate_generations_per_second(1);
        assert_eq!(result, 1000.0);
    }

    #[test]
    fn test_calculate_generations_per_second_no_time_elapsed() {
        let start = Instant::now();
        let ui = NcursesUI {
            start_time: start,
            last_generation: 0,
            last_update_time: start, // Exactly the same time
        };

        // Should return 0.0 to avoid division by zero
        let result = ui.calculate_generations_per_second(5);
        assert_eq!(result, 0.0);
    }
}
