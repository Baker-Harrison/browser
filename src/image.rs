//! Image loading and rendering module
//!
//! This module provides support for loading, decoding, and rendering common image formats
//! including PNG, JPEG, and GIF. It integrates with the display list for compositing.

use crate::error::{BrowserError, Result};
use std::sync::Arc;

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Unknown,
}

impl ImageFormat {
    /// Detect image format from magic bytes
    pub fn from_bytes(data: &[u8]) -> Self {
        if data.len() < 3 {
            return ImageFormat::Unknown;
        }

        // PNG magic bytes: 89 50 4E 47 0D 0A 1A 0A
        if data.len() >= 8
            && data[0] == 0x89
            && data[1] == 0x50
            && data[2] == 0x4E
            && data[3] == 0x47
            && data[4] == 0x0D
            && data[5] == 0x0A
            && data[6] == 0x1A
            && data[7] == 0x0A
        {
            return ImageFormat::Png;
        }

        // JPEG magic bytes: FF D8 FF
        if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
            return ImageFormat::Jpeg;
        }

        // GIF magic bytes: GIF87a or GIF89a
        if data.len() >= 6 && &data[0..3] == b"GIF" {
            return ImageFormat::Gif;
        }

        ImageFormat::Unknown
    }

    /// Get MIME type for the format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Gif => "image/gif",
            ImageFormat::Unknown => "application/octet-stream",
        }
    }
}

/// Decoded image data
#[derive(Debug, Clone)]
pub struct DecodedImage {
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Pixel data in RGBA format (8 bits per channel)
    pub data: Arc<Vec<u8>>,
    /// Original format
    pub format: ImageFormat,
}

impl DecodedImage {
    /// Create a new decoded image
    pub fn new(width: u32, height: u32, data: Vec<u8>, format: ImageFormat) -> Self {
        DecodedImage {
            width,
            height,
            data: Arc::new(data),
            format,
        }
    }

    /// Get the total number of pixels
    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Get the size of the data buffer in bytes
    pub fn data_size(&self) -> usize {
        self.data.len()
    }

    /// Check if the image is valid (has correct data size)
    pub fn is_valid(&self) -> bool {
        let expected_size = self.pixel_count() * 4; // 4 bytes per pixel (RGBA)
        self.data.len() == expected_size
    }

    /// Get a pixel at the given coordinates (returns RGBA)
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let index = ((y * self.width + x) * 4) as usize;
        if index + 3 >= self.data.len() {
            return None;
        }

        Some((
            self.data[index],
            self.data[index + 1],
            self.data[index + 2],
            self.data[index + 3],
        ))
    }
}

/// Image loader for decoding image data
pub struct ImageLoader;

impl ImageLoader {
    /// Load and decode an image from raw bytes
    ///
    /// Automatically detects the format and decodes the image.
    /// Returns an error if the format is unsupported or decoding fails.
    pub fn load(data: &[u8]) -> Result<DecodedImage> {
        let format = ImageFormat::from_bytes(data);

        match format {
            ImageFormat::Unknown => Err(BrowserError::UnsupportedImageFormat(
                "Unknown or unsupported image format".to_string(),
            )),
            ImageFormat::Png => Self::decode_png(data),
            ImageFormat::Jpeg => Self::decode_jpeg(data),
            ImageFormat::Gif => Self::decode_gif(data),
        }
    }

    /// Decode a PNG image
    fn decode_png(data: &[u8]) -> Result<DecodedImage> {
        let image = image::load_from_memory(data)
            .map_err(|e| BrowserError::ImageDecodeError(format!("PNG decode error: {}", e)))?;

        let rgba = image.to_rgba8();
        Ok(DecodedImage::new(
            rgba.width(),
            rgba.height(),
            rgba.into_raw(),
            ImageFormat::Png,
        ))
    }

    /// Decode a JPEG image
    fn decode_jpeg(data: &[u8]) -> Result<DecodedImage> {
        let image = image::load_from_memory(data)
            .map_err(|e| BrowserError::ImageDecodeError(format!("JPEG decode error: {}", e)))?;

        let rgba = image.to_rgba8();
        Ok(DecodedImage::new(
            rgba.width(),
            rgba.height(),
            rgba.into_raw(),
            ImageFormat::Jpeg,
        ))
    }

    /// Decode a GIF image (first frame only for now)
    fn decode_gif(data: &[u8]) -> Result<DecodedImage> {
        let image = image::load_from_memory(data)
            .map_err(|e| BrowserError::ImageDecodeError(format!("GIF decode error: {}", e)))?;

        let rgba = image.to_rgba8();
        Ok(DecodedImage::new(
            rgba.width(),
            rgba.height(),
            rgba.into_raw(),
            ImageFormat::Gif,
        ))
    }
}

/// Image scaling modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleMode {
    /// Stretch to fit (may distort aspect ratio)
    Stretch,
    /// Fit within bounds, preserving aspect ratio
    Fit,
    /// Fill bounds, preserving aspect ratio (may crop)
    Cover,
    /// No scaling, use original size
    None,
}

/// Image positioning and scaling options
#[derive(Debug, Clone)]
pub struct ImageOptions {
    /// How to scale the image
    pub scale_mode: ScaleMode,
    /// Horizontal alignment (for Fit/Cover modes)
    pub align_x: f32, // 0.0 = left, 0.5 = center, 1.0 = right
    /// Vertical alignment (for Fit/Cover modes)
    pub align_y: f32, // 0.0 = top, 0.5 = center, 1.0 = bottom
}

impl Default for ImageOptions {
    fn default() -> Self {
        ImageOptions {
            scale_mode: ScaleMode::Fit,
            align_x: 0.5,
            align_y: 0.5,
        }
    }
}

/// Calculate the destination rectangle for an image given bounds and options
pub fn calculate_image_rect(
    image: &DecodedImage,
    bounds: (u32, u32, u32, u32), // (x, y, width, height)
    options: &ImageOptions,
) -> (u32, u32, u32, u32) {
    let (bx, by, bw, bh) = bounds;
    let iw = image.width;
    let ih = image.height;

    match options.scale_mode {
        ScaleMode::Stretch => (bx, by, bw, bh),
        ScaleMode::None => {
            // Don't scale, clip if larger
            let w = iw.min(bw);
            let h = ih.min(bh);
            let x = bx + ((bw - w) as f32 * options.align_x) as u32;
            let y = by + ((bh - h) as f32 * options.align_y) as u32;
            (x, y, w, h)
        }
        ScaleMode::Fit => {
            // Scale to fit within bounds, preserving aspect ratio
            let scale_x = bw as f32 / iw as f32;
            let scale_y = bh as f32 / ih as f32;
            let scale = scale_x.min(scale_y);

            let w = (iw as f32 * scale) as u32;
            let h = (ih as f32 * scale) as u32;
            let x = bx + ((bw - w) as f32 * options.align_x) as u32;
            let y = by + ((bh - h) as f32 * options.align_y) as u32;
            (x, y, w, h)
        }
        ScaleMode::Cover => {
            // Scale to cover bounds, preserving aspect ratio (may crop)
            let scale_x = bw as f32 / iw as f32;
            let scale_y = bh as f32 / ih as f32;
            let scale = scale_x.max(scale_y);

            let w = (iw as f32 * scale) as u32;
            let h = (ih as f32 * scale) as u32;

            let x_offset = ((w - bw) as f32 * options.align_x) as u32;
            let y_offset = ((h - bh) as f32 * options.align_y) as u32;

            let x = if bx >= x_offset { bx - x_offset } else { 0 };
            let y = if by >= y_offset { by - y_offset } else { 0 };

            (x, y, w, h)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection_png() {
        // PNG magic bytes
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        let format = ImageFormat::from_bytes(&png_header);
        assert_eq!(format, ImageFormat::Png);
        assert_eq!(format.mime_type(), "image/png");
    }

    #[test]
    fn test_format_detection_jpeg() {
        // JPEG magic bytes
        let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        let format = ImageFormat::from_bytes(&jpeg_header);
        assert_eq!(format, ImageFormat::Jpeg);
        assert_eq!(format.mime_type(), "image/jpeg");
    }

    #[test]
    fn test_format_detection_gif() {
        // GIF magic bytes
        let gif_header = b"GIF89a";
        let format = ImageFormat::from_bytes(gif_header);
        assert_eq!(format, ImageFormat::Gif);
        assert_eq!(format.mime_type(), "image/gif");
    }

    #[test]
    fn test_format_detection_unknown() {
        let unknown = [0x00, 0x01, 0x02, 0x03];
        let format = ImageFormat::from_bytes(&unknown);
        assert_eq!(format, ImageFormat::Unknown);
    }

    #[test]
    fn test_format_detection_too_short() {
        let short = [0x00, 0x01];
        let format = ImageFormat::from_bytes(&short);
        assert_eq!(format, ImageFormat::Unknown);
    }

    #[test]
    fn test_decoded_image_creation() {
        let data = vec![255u8; 400]; // 10x10 RGBA
        let image = DecodedImage::new(10, 10, data, ImageFormat::Png);
        assert_eq!(image.width, 10);
        assert_eq!(image.height, 10);
        assert_eq!(image.pixel_count(), 100);
        assert_eq!(image.data_size(), 400);
        assert!(image.is_valid());
    }

    #[test]
    fn test_decoded_image_invalid() {
        let data = vec![0u8; 50]; // Wrong size for 10x10 RGBA
        let image = DecodedImage::new(10, 10, data, ImageFormat::Png);
        assert!(!image.is_valid());
    }

    #[test]
    fn test_get_pixel() {
        let mut data = vec![0u8; 16]; // 2x2 RGBA
        data[0] = 255; // R at (0,0)
        data[1] = 128; // G at (0,0)
        data[2] = 64; // B at (0,0)
        data[3] = 255; // A at (0,0)

        let image = DecodedImage::new(2, 2, data, ImageFormat::Png);
        let pixel = image.get_pixel(0, 0);
        assert_eq!(pixel, Some((255, 128, 64, 255)));

        let pixel = image.get_pixel(1, 1);
        assert_eq!(pixel, Some((0, 0, 0, 0)));

        // Out of bounds
        assert_eq!(image.get_pixel(2, 0), None);
        assert_eq!(image.get_pixel(0, 2), None);
    }

    #[test]
    fn test_image_options_default() {
        let opts = ImageOptions::default();
        assert_eq!(opts.scale_mode, ScaleMode::Fit);
        assert_eq!(opts.align_x, 0.5);
        assert_eq!(opts.align_y, 0.5);
    }

    #[test]
    fn test_calculate_image_rect_stretch() {
        let image = DecodedImage::new(100, 50, vec![0u8; 20000], ImageFormat::Png);
        let opts = ImageOptions {
            scale_mode: ScaleMode::Stretch,
            ..Default::default()
        };

        let rect = calculate_image_rect(&image, (10, 10, 200, 100), &opts);
        assert_eq!(rect, (10, 10, 200, 100));
    }

    #[test]
    fn test_calculate_image_rect_none() {
        let image = DecodedImage::new(100, 50, vec![0u8; 20000], ImageFormat::Png);
        let opts = ImageOptions {
            scale_mode: ScaleMode::None,
            ..Default::default()
        };

        // Fits within bounds
        let rect = calculate_image_rect(&image, (10, 10, 200, 100), &opts);
        assert_eq!(rect, (60, 35, 100, 50)); // Centered

        // Larger than bounds, should clip
        let rect = calculate_image_rect(&image, (10, 10, 50, 25), &opts);
        assert_eq!(rect, (10, 10, 50, 25)); // Clipped to bounds
    }

    #[test]
    fn test_calculate_image_rect_fit() {
        let image = DecodedImage::new(100, 50, vec![0u8; 20000], ImageFormat::Png);
        let opts = ImageOptions {
            scale_mode: ScaleMode::Fit,
            ..Default::default()
        };

        // Fit into larger bounds
        let rect = calculate_image_rect(&image, (0, 0, 200, 200), &opts);
        assert_eq!(rect, (0, 50, 200, 100)); // Scaled and centered

        // Fit into smaller bounds
        let rect = calculate_image_rect(&image, (0, 0, 50, 50), &opts);
        assert_eq!(rect, (0, 12, 50, 25)); // Scaled down and centered
    }

    #[test]
    fn test_calculate_image_rect_cover() {
        let image = DecodedImage::new(100, 50, vec![0u8; 20000], ImageFormat::Png);
        let opts = ImageOptions {
            scale_mode: ScaleMode::Cover,
            ..Default::default()
        };

        let rect = calculate_image_rect(&image, (0, 0, 50, 50), &opts);
        // Should scale to cover, resulting in larger rect
        assert!(rect.2 >= 50);
        assert!(rect.3 >= 50);
    }

    #[test]
    fn test_calculate_image_rect_alignment() {
        let image = DecodedImage::new(100, 50, vec![0u8; 20000], ImageFormat::Png);
        let opts = ImageOptions {
            scale_mode: ScaleMode::Fit,
            align_x: 0.0,
            align_y: 0.0,
        };

        let rect = calculate_image_rect(&image, (0, 0, 200, 200), &opts);
        assert_eq!(rect, (0, 0, 200, 100)); // Top-left aligned
    }

    #[test]
    fn test_load_invalid_format() {
        let invalid_data = [0x00, 0x01, 0x02, 0x03];
        let result = ImageLoader::load(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_empty_data() {
        let result = ImageLoader::load(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_simple_png() {
        // Create a minimal valid PNG (1x1 red pixel)
        // This is a simplified test - in real scenarios, you'd use actual PNG files
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let format = ImageFormat::from_bytes(&png_header);
        assert_eq!(format, ImageFormat::Png);
    }
}
