use image::{ImageBuffer, Luma};
use rusttype::{Font, Scale, point};
use std::collections::HashMap;

/// Generator for ASCII art that converts characters to image buffers and manages character rendering
pub struct AsciiGenerator {
    font: Font<'static>,
    scale: Scale,
    char_width: u32,
    char_height: u32,
    char_cache: HashMap<u8, ImageBuffer<Luma<u8>, Vec<u8>>>,
}

impl AsciiGenerator {
    /// Creates a new ASCII generator with a monospace font at 12pt
    pub fn new() -> Self {
        let font = Self::load_font();
        
        let scale = Scale::uniform(12.0);
        
        // Calculate character dimensions for monospace font
        let glyph = font.glyph('M').scaled(scale);
        let h_metrics = glyph.h_metrics();
        let char_width = h_metrics.advance_width.ceil() as u32;
        let char_height = (scale.y * 1.2).ceil() as u32; // Add line spacing
        
        let mut generator = Self {
            font,
            scale,
            char_width,
            char_height,
            char_cache: HashMap::new(),
        };
        
        // Pre-cache all ASCII characters from 0x20 to 0x7F
        generator.build_char_cache();
        generator
    }
    
    /// Loads the font, with fallback for testing
    fn load_font() -> Font<'static> {
        // Use embedded font data
        let font_data = include_bytes!("../assets/DejaVuSansMono.ttf");
        Font::try_from_bytes(font_data as &[u8])
            .expect("Failed to load embedded font")
    }
    
    /// Pre-renders all 7-bit ASCII characters starting at 0x20 and caches them
    fn build_char_cache(&mut self) {
        for ascii_code in 0x20..=0x7F {
            let char_img = self.render_char(ascii_code as char);
            self.char_cache.insert(ascii_code, char_img);
        }
    }
    
    /// Renders a single character to a grayscale image buffer
    fn render_char(&self, ch: char) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let mut img = ImageBuffer::new(self.char_width, self.char_height);
        
        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = Luma([255u8]);
        }
        
        let glyph = self.font.glyph(ch).scaled(self.scale);
        
        if let Some(bb) = glyph.exact_bounding_box() {
            let positioned_glyph = glyph.positioned(point(0.0, self.scale.y));
            
            positioned_glyph.draw(|x, y, v| {
                let px = x as i32 + bb.min.x as i32;
                let py = y as i32 + bb.min.y as i32;
                
                if px >= 0 && py >= 0 && (px as u32) < self.char_width && (py as u32) < self.char_height {
                    let intensity = (255.0 * (1.0 - v)) as u8;
                    img.put_pixel(px as u32, py as u32, Luma([intensity]));
                }
            });
        }
        
        img
    }
    
    /// Generates an ASCII art image buffer from a vector of character codes
    pub fn generate_ascii_image(&self, chars: &[u8], width: u32, height: u32) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let img_width = width * self.char_width;
        let img_height = height * self.char_height;
        let mut result = ImageBuffer::new(img_width, img_height);
        
        // Fill with white background
        for pixel in result.pixels_mut() {
            *pixel = Luma([255u8]);
        }
        
        for (i, &char_code) in chars.iter().enumerate() {
            let x = (i as u32) % width;
            let y = (i as u32) / width;
            
            if y >= height {
                break;
            }
            
            if let Some(char_img) = self.char_cache.get(&char_code) {
                self.copy_char_to_image(&mut result, char_img, x * self.char_width, y * self.char_height);
            }
        }
        
        result
    }
    
    /// Copies a character image to a specific position in the target image
    fn copy_char_to_image(
        &self,
        target: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
        char_img: &ImageBuffer<Luma<u8>, Vec<u8>>,
        start_x: u32,
        start_y: u32,
    ) {
        for y in 0..self.char_height {
            for x in 0..self.char_width {
                if start_x + x < target.width() && start_y + y < target.height() {
                    let pixel = char_img.get_pixel(x, y);
                    target.put_pixel(start_x + x, start_y + y, *pixel);
                }
            }
        }
    }
    
    /// Converts a vector of characters to a readable string representation
    pub fn individual_to_string(&self, individual: &crate::genetic_algorithm::Individual, width: u32) -> String {
        let mut result = String::new();
        
        for (i, &char_code) in individual.chars.iter().enumerate() {
            if i > 0 && (i as u32) % width == 0 {
                result.push('\n');
            }
            result.push(char_code as char);
        }
        
        result
    }
    
    /// Returns the dimensions of a single character in pixels
    #[cfg(test)]
    pub fn char_dimensions(&self) -> (u32, u32) {
        (self.char_width, self.char_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_generator_creation() {
        let generator = AsciiGenerator::new();
        assert!(!generator.char_cache.is_empty());
        assert!(generator.char_cache.len() >= 95); // 0x20 to 0x7F
    }
    
    #[test]
    fn test_char_dimensions() {
        let generator = AsciiGenerator::new();
        let (width, height) = generator.char_dimensions();
        assert!(width > 0);
        assert!(height > 0);
    }
    
    #[test]
    fn test_generate_ascii_image() {
        let generator = AsciiGenerator::new();
        let chars = vec![b'A', b'B', b'C', b'D'];
        let result = generator.generate_ascii_image(&chars, 2, 2);
        
        let (char_width, char_height) = generator.char_dimensions();
        assert_eq!(result.width(), 2 * char_width);
        assert_eq!(result.height(), 2 * char_height);
    }
    
    #[test]
    fn test_individual_to_string() {
        let generator = AsciiGenerator::new();
        let individual = crate::genetic_algorithm::Individual {
            chars: vec![b'H', b'i', b'!', b' '],
            fitness: 0.0,
        };
        
        let result = generator.individual_to_string(&individual, 2);
        assert_eq!(result, "Hi\n! ");
    }
    
    #[test]
    fn test_render_char() {
        let generator = AsciiGenerator::new();
        let char_img = generator.render_char('A');
        
        let (char_width, char_height) = generator.char_dimensions();
        assert_eq!(char_img.width(), char_width);
        assert_eq!(char_img.height(), char_height);
    }
}