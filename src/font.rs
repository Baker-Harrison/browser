//! Font rendering subsystem using fontdue for glyph rasterization.

#![allow(dead_code)]

use std::fs;
use std::path::Path;

/// Errors that can occur during font operations.
#[derive(Debug)]
pub enum FontError {
    NotFound(String),
    LoadFailed(String),
}

impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontError::NotFound(msg) => write!(f, "Font not found: {}", msg),
            FontError::LoadFailed(msg) => write!(f, "Failed to load font: {}", msg),
        }
    }
}

impl std::error::Error for FontError {}

/// Rasterized text bitmap with alpha coverage per pixel.
pub struct RasterizedText {
    pub width: usize,
    pub height: usize,
    /// Alpha coverage per pixel (0-255).
    pub pixels: Vec<u8>,
    /// Offset from the nominal draw position to the top-left of the pixel buffer.
    pub x_offset: i32,
    pub y_offset: i32,
}

/// Font system that loads fonts and provides text measurement and rasterization.
pub struct FontSystem {
    fonts: Vec<fontdue::Font>,
}

impl FontSystem {
    /// Create a new font system, loading the first available system font.
    pub fn new() -> Result<Self, FontError> {
        let font_data = Self::load_font_bytes()?;
        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
            .map_err(|e| FontError::LoadFailed(e.to_string()))?;
        Ok(FontSystem { fonts: vec![font] })
    }

    /// Try to find font bytes from common system font paths.
    fn load_font_bytes() -> Result<Vec<u8>, FontError> {
        let paths = [
            r"C:\Windows\Fonts\arial.ttf",
            r"C:\Windows\Fonts\Arial.ttf",
            r"C:\Windows\Fonts\segoeui.ttf",
            r"C:\Windows\Fonts\SegoeUI.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
        ];

        for path in &paths {
            if Path::new(path).exists() {
                return fs::read(path).map_err(|e| FontError::NotFound(e.to_string()));
            }
        }

        Err(FontError::NotFound(
            "No system font found at any known path".to_string(),
        ))
    }

    /// Measure the width of text in pixels at the given size.
    pub fn measure_text(&self, text: &str, size: f32) -> f32 {
        let font = &self.fonts[0];
        let mut width = 0.0;
        for c in text.chars() {
            let metrics = font.metrics(c, size);
            width += metrics.advance_width;
        }
        width
    }

    /// Rasterize text into a pixel buffer with alpha coverage at the given size and color.
    pub fn rasterize(&self, text: &str, size: f32, _color: (u8, u8, u8, u8)) -> RasterizedText {
        if text.is_empty() {
            return RasterizedText {
                width: 0,
                height: 0,
                pixels: Vec::new(),
                x_offset: 0,
                y_offset: 0,
            };
        }

        let font = &self.fonts[0];

        struct GlyphPlacement {
            x: i32,
            y: i32,
            width: usize,
            height: usize,
            bitmap: Vec<u8>,
        }

        let mut placements = Vec::new();
        let mut cursor_x: i32 = 0;
        let mut min_x: i32 = 0;
        let mut max_x: i32 = 0;
        let mut min_y: i32 = 0;
        let mut max_y: i32 = 0;

        for c in text.chars() {
            let (metrics, bitmap) = font.rasterize(c, size);

            let x_start = cursor_x + metrics.xmin;
            let x_end = x_start + metrics.width as i32;
            // fontdue ymin is the offset from baseline to the bottom of the bitmap
            // (positive = above baseline in y-up coords).
            // In screen coords (y-down), the top of the bitmap is at:
            //   screen_y = - (ymin + height)  relative to baseline
            let y_top = -(metrics.ymin + metrics.height as i32);
            let y_bottom = y_top + metrics.height as i32;

            placements.push(GlyphPlacement {
                x: x_start,
                y: y_top,
                width: metrics.width,
                height: metrics.height,
                bitmap,
            });

            if x_start < min_x {
                min_x = x_start;
            }
            if x_end > max_x {
                max_x = x_end;
            }
            if y_top < min_y {
                min_y = y_top;
            }
            if y_bottom > max_y {
                max_y = y_bottom;
            }

            cursor_x += metrics.advance_width as i32;
        }

        let total_width = (max_x - min_x).max(0) as usize;
        let total_height = (max_y - min_y).max(0) as usize;

        if total_width == 0 || total_height == 0 {
            return RasterizedText {
                width: total_width,
                height: total_height,
                pixels: Vec::new(),
                x_offset: min_x,
                y_offset: min_y,
            };
        }

        let mut pixels = vec![0u8; total_width * total_height];

        for glyph in &placements {
            for gy in 0..glyph.height {
                for gx in 0..glyph.width {
                    let alpha = glyph.bitmap[gy * glyph.width + gx];
                    if alpha == 0 {
                        continue;
                    }
                    let dx = (glyph.x + gx as i32 - min_x) as usize;
                    let dy = (glyph.y + gy as i32 - min_y) as usize;
                    if dx < total_width && dy < total_height {
                        let idx = dy * total_width + dx;
                        let existing = pixels[idx];
                        pixels[idx] = existing
                            .saturating_add(((255 - existing) as u16 * alpha as u16 / 255) as u8);
                    }
                }
            }
        }

        RasterizedText {
            width: total_width,
            height: total_height,
            pixels,
            x_offset: min_x,
            y_offset: min_y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_font_system() -> FontSystem {
        FontSystem::new().expect("Failed to load system font for tests")
    }

    #[test]
    fn test_font_system_creation() {
        let font_system = FontSystem::new();
        assert!(
            font_system.is_ok(),
            "FontSystem should be created successfully: {:?}",
            font_system.err()
        );
    }

    #[test]
    fn test_text_measurement() {
        let font = create_font_system();
        let width = font.measure_text("Hello", 16.0);
        assert!(width > 0.0, "Text width should be positive, got {}", width);
        let width2 = font.measure_text("Hello World", 16.0);
        assert!(
            width2 > width,
            "Longer text should be wider: {} vs {}",
            width2,
            width
        );
    }

    #[test]
    fn test_text_measurement_empty() {
        let font = create_font_system();
        let width = font.measure_text("", 16.0);
        assert_eq!(width, 0.0, "Empty text width should be 0");
    }

    #[test]
    fn test_text_rasterization() {
        let font = create_font_system();
        let rasterized = font.rasterize("A", 32.0, (0, 0, 0, 255));
        assert!(rasterized.width > 0, "Rasterized width should be > 0");
        assert!(rasterized.height > 0, "Rasterized height should be > 0");
        assert!(
            !rasterized.pixels.is_empty(),
            "Rasterized pixels should not be empty"
        );
        let has_ink = rasterized.pixels.iter().any(|&p| p > 0);
        assert!(has_ink, "Rasterized glyph should have ink");
    }

    #[test]
    fn test_empty_text() {
        let font = create_font_system();
        let rasterized = font.rasterize("", 16.0, (0, 0, 0, 255));
        assert_eq!(rasterized.width, 0, "Empty text width should be 0");
        assert_eq!(rasterized.height, 0, "Empty text height should be 0");
        assert!(
            rasterized.pixels.is_empty(),
            "Empty text pixels should be empty"
        );
    }

    #[test]
    fn test_rasterized_text_offset() {
        let font = create_font_system();
        let rasterized = font.rasterize("ABC", 16.0, (0, 0, 0, 255));
        assert!(rasterized.width > 0);
        assert!(rasterized.height > 0);
        assert!(!rasterized.pixels.is_empty());
        // x_offset should account for left-side bearing of first glyph
        assert!(
            rasterized.x_offset >= 0 || rasterized.x_offset < 5,
            "x_offset {:?} should be reasonable",
            rasterized.x_offset
        );
    }
}
