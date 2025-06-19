use image::{DynamicImage, ImageBuffer, Luma, ImageError};
use fast_image_resize as fir;
use fast_image_resize::images::Image;
use std::path::Path;

pub struct ImageProcessor;

impl ImageProcessor {
    /// Creates a new ImageProcessor instance
    pub fn new() -> Self {
        Self
    }
    
    /// Loads an image from the specified file path
    pub fn load_image<P: AsRef<Path>>(&self, path: P) -> Result<DynamicImage, ImageError> {
        image::open(path)
    }
    
    /// Prepares target image by resizing and converting to grayscale
    /// This creates the reference image that the genetic algorithm will try to match
    pub fn prepare_target_image(
        &self,
        img: &DynamicImage,
        target_width: u32,
        target_height: u32,
    ) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
        let resized = self.resize_image(img, target_width, target_height)?;
        Ok(self.convert_to_grayscale(&resized))
    }
    
    /// Resizes an image to the specified dimensions using high-quality Lanczos3 filtering
    fn resize_image(
        &self,
        img: &DynamicImage,
        target_width: u32,
        target_height: u32,
    ) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        let src_image = Image::from_vec_u8(
            img.width(),
            img.height(),
            img.to_rgb8().into_raw(),
            fir::PixelType::U8x3,
        )?;
        
        let mut dst_image = Image::new(
            target_width,
            target_height,
            fir::PixelType::U8x3,
        );
        
        let mut resizer = fir::Resizer::new();
        resizer.resize(&src_image, &mut dst_image, &fir::ResizeOptions::new())?;
        
        let resized_buffer = image::RgbImage::from_raw(
            target_width,
            target_height,
            dst_image.into_vec(),
        ).ok_or("Failed to create RGB image from resized buffer")?;
        
        Ok(DynamicImage::ImageRgb8(resized_buffer))
    }
    
    /// Converts a color image to grayscale for easier comparison with ASCII art
    fn convert_to_grayscale(&self, img: &DynamicImage) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        img.to_luma8()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbImage, DynamicImage};

    #[test]
    fn test_convert_to_grayscale() {
        let processor = ImageProcessor::new();
        let rgb_img = RgbImage::new(10, 10);
        let dynamic_img = DynamicImage::ImageRgb8(rgb_img);
        
        let gray_img = processor.convert_to_grayscale(&dynamic_img);
        assert_eq!(gray_img.width(), 10);
        assert_eq!(gray_img.height(), 10);
    }
    
    #[test]
    fn test_resize_image() {
        let processor = ImageProcessor::new();
        let rgb_img = RgbImage::new(100, 100);
        let dynamic_img = DynamicImage::ImageRgb8(rgb_img);
        
        let resized = processor.resize_image(&dynamic_img, 50, 50).unwrap();
        assert_eq!(resized.width(), 50);
        assert_eq!(resized.height(), 50);
    }
    
    #[test]
    fn test_prepare_target_image() {
        let processor = ImageProcessor::new();
        let rgb_img = RgbImage::new(100, 100);
        let dynamic_img = DynamicImage::ImageRgb8(rgb_img);
        
        let result = processor.prepare_target_image(&dynamic_img, 50, 50).unwrap();
        assert_eq!(result.width(), 50);
        assert_eq!(result.height(), 50);
    }
}