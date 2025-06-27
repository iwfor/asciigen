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

    /// Prepares target image with optional inversion, resizing and converting to grayscale
    /// This creates the reference image that the genetic algorithm will try to match
    pub fn prepare_target_image_with_inversion(
        &self,
        img: &DynamicImage,
        target_width: u32,
        target_height: u32,
        invert: bool,
    ) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
        let resized = self.resize_image(img, target_width, target_height)?;
        let mut grayscale = self.convert_to_grayscale(&resized);
        
        if invert {
            self.invert_image(&mut grayscale);
        }
        
        Ok(grayscale)
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

    /// Inverts a grayscale image (255 - pixel_value for each pixel)
    fn invert_image(&self, img: &mut ImageBuffer<Luma<u8>, Vec<u8>>) {
        for pixel in img.pixels_mut() {
            pixel[0] = 255 - pixel[0];
        }
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
    fn test_invert_image() {
        let processor = ImageProcessor::new();
        let mut img = ImageBuffer::new(2, 2);
        
        // Set some test pixel values
        img.put_pixel(0, 0, Luma([100]));
        img.put_pixel(1, 0, Luma([200]));
        img.put_pixel(0, 1, Luma([0]));
        img.put_pixel(1, 1, Luma([255]));

        processor.invert_image(&mut img);

        // Check that pixels are inverted (255 - original_value)
        assert_eq!(img.get_pixel(0, 0)[0], 155); // 255 - 100
        assert_eq!(img.get_pixel(1, 0)[0], 55);  // 255 - 200
        assert_eq!(img.get_pixel(0, 1)[0], 255); // 255 - 0
        assert_eq!(img.get_pixel(1, 1)[0], 0);   // 255 - 255
    }

    #[test]
    fn test_prepare_target_image_with_inversion() {
        let processor = ImageProcessor::new();
        let rgb_img = RgbImage::new(10, 10);
        let dynamic_img = DynamicImage::ImageRgb8(rgb_img);

        // Test without inversion
        let result_normal = processor.prepare_target_image_with_inversion(&dynamic_img, 5, 5, false).unwrap();
        assert_eq!(result_normal.width(), 5);
        assert_eq!(result_normal.height(), 5);

        // Test with inversion
        let result_inverted = processor.prepare_target_image_with_inversion(&dynamic_img, 5, 5, true).unwrap();
        assert_eq!(result_inverted.width(), 5);
        assert_eq!(result_inverted.height(), 5);

        // The inverted result should have different pixel values than the normal result
        // (unless the original image was perfectly gray at 127.5, which is unlikely with a blank image)
        let normal_pixel = result_normal.get_pixel(0, 0)[0];
        let inverted_pixel = result_inverted.get_pixel(0, 0)[0];
        assert_eq!(normal_pixel + inverted_pixel, 255); // Should sum to 255 due to inversion
    }
}
