use ncurses::*;
use std::time::{Duration, Instant};

/// Interactive ncurses UI for displaying genetic algorithm progress
pub struct NcursesUI {
    last_generation_time: Instant,
    generation_times: Vec<Duration>,
    max_generation_history: usize,
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
            last_generation_time: Instant::now(),
            generation_times: Vec::new(),
            max_generation_history: 100, // Keep last 100 generation times for averaging
        })
    }
    
    /// Update the display with current statistics
    pub fn update(&mut self, stats: &UIStats) {
        // Record generation timing
        let now = Instant::now();
        let generation_duration = now.duration_since(self.last_generation_time);
        self.generation_times.push(generation_duration);
        
        // Keep only recent generation times
        if self.generation_times.len() > self.max_generation_history {
            self.generation_times.remove(0);
        }
        
        self.last_generation_time = now;
        
        // Clear screen and reset cursor
        clear();
        mv(0, 0);
        
        // Draw header
        self.draw_header();
        
        // Draw main statistics
        self.draw_stats(stats);
        
        // Draw progress bar
        self.draw_progress_bar(stats.generation, stats.total_generations);
        
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
        
        // Generation info
        attron(COLOR_PAIR(5)); // White for labels
        mvprintw(y_start, 0, "Generation:");
        attroff(COLOR_PAIR(5));
        attron(COLOR_PAIR(1)); // Green for values
        mvprintw(y_start, 15, &format!("{}/{}", stats.generation, stats.total_generations));
        attroff(COLOR_PAIR(1));
        
        // Progress percentage
        let progress = (stats.generation as f64 / stats.total_generations as f64) * 100.0;
        attron(COLOR_PAIR(5));
        mvprintw(y_start, 35, "Progress:");
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
        let gens_per_sec = self.calculate_generations_per_second();
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
        
        // ETA (Estimated Time of Arrival)
        if stats.generation > 0 && gens_per_sec > 0.0 {
            let remaining_gens = stats.total_generations - stats.generation;
            let eta_seconds = remaining_gens as f64 / gens_per_sec;
            attron(COLOR_PAIR(5));
            mvprintw(y_start + 3, 35, "ETA:");
            attroff(COLOR_PAIR(5));
            attron(COLOR_PAIR(2));
            mvprintw(y_start + 3, 40, &format!("{:.1}s", eta_seconds));
            attroff(COLOR_PAIR(2));
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
    
    /// Draw ASCII art if provided
    fn draw_ascii_art(&self, art: &str) {
        let y_start = 11;
        
        attron(COLOR_PAIR(4));
        mvprintw(y_start, 0, "Current Best ASCII Art:");
        attroff(COLOR_PAIR(4));
        
        attron(COLOR_PAIR(5));
        for (i, line) in art.lines().enumerate() {
            mvprintw(y_start + 2 + i as i32, 0, line);
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
    
    /// Calculate average generations per second
    fn calculate_generations_per_second(&self) -> f64 {
        if self.generation_times.is_empty() {
            return 0.0;
        }
        
        let total_duration: Duration = self.generation_times.iter().sum();
        let avg_duration = total_duration / self.generation_times.len() as u32;
        
        if avg_duration.as_secs_f64() > 0.0 {
            1.0 / avg_duration.as_secs_f64()
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