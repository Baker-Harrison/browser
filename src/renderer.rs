//! 2D rendering module for browser chrome
//!
//! This module handles basic 2D rendering using softbuffer for drawing
//! browser UI elements like address bar and navigation buttons.

use crate::font::FontSystem;

/// Color represented as RGBA
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    #[allow(dead_code)]
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

    #[allow(dead_code)]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    /// Convert to u32 for softbuffer (0x00RRGGBB format)
    pub fn to_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

// Common colors
pub const WHITE: Color = Color::new(255, 255, 255);
pub const BLACK: Color = Color::new(0, 0, 0);
pub const GRAY: Color = Color::new(200, 200, 200);
pub const DARK_GRAY: Color = Color::new(100, 100, 100);
pub const BLUE: Color = Color::new(70, 130, 180);
#[allow(dead_code)]
pub const LIGHT_BLUE: Color = Color::new(135, 206, 250);

/// Rectangle for drawing operations
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    #[allow(dead_code)]
    /// Check if a point is inside the rectangle
    pub fn contains(&self, px: u32, py: u32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }
}

/// Browser chrome layout constants
pub mod layout {
    use super::Rect;

    pub const CHROME_HEIGHT: u32 = 40;
    pub const BUTTON_SIZE: u32 = 30;
    pub const BUTTON_SPACING: u32 = 5;
    pub const ADDRESS_BAR_WIDTH: u32 = 400;
    pub const ADDRESS_BAR_HEIGHT: u32 = 30;

    /// Back button rectangle
    pub fn back_button() -> Rect {
        Rect::new(BUTTON_SPACING, BUTTON_SPACING, BUTTON_SIZE, BUTTON_SIZE)
    }

    /// Forward button rectangle
    pub fn forward_button() -> Rect {
        Rect::new(
            BUTTON_SPACING * 2 + BUTTON_SIZE,
            BUTTON_SPACING,
            BUTTON_SIZE,
            BUTTON_SIZE,
        )
    }

    /// Address bar rectangle
    pub fn address_bar() -> Rect {
        Rect::new(
            BUTTON_SPACING * 3 + BUTTON_SIZE * 2,
            BUTTON_SPACING,
            ADDRESS_BAR_WIDTH,
            ADDRESS_BAR_HEIGHT,
        )
    }

    /// Go button rectangle
    pub fn go_button() -> Rect {
        Rect::new(
            BUTTON_SPACING * 4 + BUTTON_SIZE * 2 + ADDRESS_BAR_WIDTH,
            BUTTON_SPACING,
            50,
            ADDRESS_BAR_HEIGHT,
        )
    }

    /// Full chrome area
    pub fn chrome_area(window_width: u32) -> Rect {
        Rect::new(0, 0, window_width, CHROME_HEIGHT)
    }

    /// Content area (below chrome)
    #[allow(dead_code)]
    pub fn content_area(window_width: u32, window_height: u32) -> Rect {
        Rect::new(
            0,
            CHROME_HEIGHT,
            window_width,
            window_height - CHROME_HEIGHT,
        )
    }
}

/// 2D renderer for browser chrome
///
/// TODO: This is a software renderer for MVP. Future work should migrate to
/// GPU-accelerated rendering (wgpu) for better performance with complex UIs.
pub struct Renderer {
    buffer: Vec<u32>,
    width: u32,
    height: u32,
}

impl Renderer {
    /// Create a new renderer with the given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let buffer = vec![WHITE.to_u32(); (width * height) as usize];
        Renderer {
            buffer,
            width,
            height,
        }
    }

    /// Get the width of the renderer
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height of the renderer
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the buffer as a slice for softbuffer
    pub fn buffer(&self) -> &[u32] {
        &self.buffer
    }

    /// Resize the renderer to new dimensions without reallocating if possible
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        let new_size = (new_width * new_height) as usize;
        let current_size = self.buffer.len();

        if new_size != current_size {
            self.buffer.resize(new_size, WHITE.to_u32());
        }

        self.width = new_width;
        self.height = new_height;
    }

    /// Clear the entire buffer with a color
    pub fn clear(&mut self, color: Color) {
        let color_u32 = color.to_u32();
        for pixel in &mut self.buffer {
            *pixel = color_u32;
        }
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let color_u32 = color.to_u32();
        let x_start = rect.x as usize;
        let y_start = rect.y as usize;
        let width = rect.width as usize;
        let height = rect.height as usize;

        for y in y_start..(y_start + height) {
            for x in x_start..(x_start + width) {
                if y < self.height as usize && x < self.width as usize {
                    let index = y * self.width as usize + x;
                    self.buffer[index] = color_u32;
                }
            }
        }
    }

    /// Draw a rectangle outline
    pub fn draw_rect(&mut self, rect: Rect, color: Color, thickness: u32) {
        let color_u32 = color.to_u32();
        let thickness = thickness as usize;

        // Top edge
        for y in rect.y as usize..(rect.y as usize + thickness) {
            for x in rect.x as usize..(rect.x as usize + rect.width as usize) {
                if y < self.height as usize && x < self.width as usize {
                    let index = y * self.width as usize + x;
                    self.buffer[index] = color_u32;
                }
            }
        }

        // Bottom edge
        let bottom_start = (rect.y + rect.height - thickness as u32) as usize;
        for y in bottom_start..(rect.y + rect.height) as usize {
            for x in rect.x as usize..(rect.x as usize + rect.width as usize) {
                if y < self.height as usize && x < self.width as usize {
                    let index = y * self.width as usize + x;
                    self.buffer[index] = color_u32;
                }
            }
        }

        // Left edge
        for y in rect.y as usize..(rect.y + rect.height) as usize {
            for x in rect.x as usize..(rect.x as usize + thickness) {
                if y < self.height as usize && x < self.width as usize {
                    let index = y * self.width as usize + x;
                    self.buffer[index] = color_u32;
                }
            }
        }

        // Right edge
        let right_start = (rect.x + rect.width - thickness as u32) as usize;
        for y in rect.y as usize..(rect.y + rect.height) as usize {
            for x in right_start..(rect.x + rect.width) as usize {
                if y < self.height as usize && x < self.width as usize {
                    let index = y * self.width as usize + x;
                    self.buffer[index] = color_u32;
                }
            }
        }
    }

    /// Draw the browser chrome UI
    pub fn draw_chrome(&mut self, window_width: u32) {
        // Draw chrome background
        let chrome_area = layout::chrome_area(window_width);
        self.fill_rect(chrome_area, GRAY);

        // Draw back button
        let back_button = layout::back_button();
        self.fill_rect(back_button, DARK_GRAY);
        self.draw_rect(back_button, BLACK, 2);

        // Draw forward button
        let forward_button = layout::forward_button();
        self.fill_rect(forward_button, DARK_GRAY);
        self.draw_rect(forward_button, BLACK, 2);

        // Draw address bar
        let address_bar = layout::address_bar();
        self.fill_rect(address_bar, WHITE);
        self.draw_rect(address_bar, DARK_GRAY, 2);

        // Draw go button
        let go_button = layout::go_button();
        self.fill_rect(go_button, BLUE);
        self.draw_rect(go_button, BLACK, 2);
    }

    #[allow(dead_code)]
    /// Draw text at the given position using the font system.
    /// (x, y) is the top-left position of the rendered text.
    pub fn draw_text(
        &mut self,
        font: &FontSystem,
        x: u32,
        y: u32,
        text: &str,
        size: f32,
        color: Color,
    ) {
        let rasterized = font.rasterize(text, size, (color.r, color.g, color.b, color.a));
        if rasterized.width == 0 || rasterized.height == 0 {
            return;
        }

        for py in 0..rasterized.height {
            for px in 0..rasterized.width {
                let alpha = rasterized.pixels[py * rasterized.width + px];
                if alpha == 0 {
                    continue;
                }
                let dest_x = x as i32 + rasterized.x_offset + px as i32;
                let dest_y = y as i32 + rasterized.y_offset + py as i32;
                if dest_x >= 0
                    && dest_y >= 0
                    && (dest_x as u32) < self.width
                    && (dest_y as u32) < self.height
                {
                    let idx = (dest_y as u32 * self.width + dest_x as u32) as usize;
                    let bg = self.buffer[idx];
                    let br = (bg >> 16) & 0xFF;
                    let bg_g = (bg >> 8) & 0xFF;
                    let bb = bg & 0xFF;
                    let a = alpha as u32;
                    let inv_a = 255 - a;
                    let r = ((color.r as u32 * a + br * inv_a) / 255) as u8;
                    let g = ((color.g as u32 * a + bg_g * inv_a) / 255) as u8;
                    let b = ((color.b as u32 * a + bb * inv_a) / 255) as u8;
                    self.buffer[idx] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::new(255, 0, 0);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_color_to_u32() {
        let color = Color::new(255, 128, 64);
        let u32_color = color.to_u32();
        assert!(u32_color > 0);
    }

    #[test]
    fn test_rect_creation() {
        let rect = Rect::new(10, 20, 100, 50);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 100);
        assert_eq!(rect.height, 50);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 20, 100, 50);
        assert!(rect.contains(15, 25));
        assert!(rect.contains(50, 30));
        assert!(!rect.contains(5, 25));
        assert!(!rect.contains(15, 15));
        assert!(!rect.contains(150, 30));
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new(800, 600);
        assert_eq!(renderer.width, 800);
        assert_eq!(renderer.height, 600);
        assert_eq!(renderer.buffer.len(), 800 * 600);
    }

    #[test]
    fn test_renderer_clear() {
        let mut renderer = Renderer::new(100, 100);
        renderer.clear(BLACK);
        assert_eq!(renderer.buffer[0], BLACK.to_u32());
        assert_eq!(renderer.buffer[9999], BLACK.to_u32());
    }

    #[test]
    fn test_layout_constants() {
        let back = layout::back_button();
        assert_eq!(back.width, layout::BUTTON_SIZE);
        assert_eq!(back.height, layout::BUTTON_SIZE);

        let forward = layout::forward_button();
        assert!(forward.x > back.x); // Forward button should be to the right of back
    }

    fn create_test_font() -> crate::font::FontSystem {
        crate::font::FontSystem::new().expect("Failed to load system font for tests")
    }

    #[test]
    fn test_renderer_draw_text() {
        let mut renderer = Renderer::new(200, 100);
        renderer.clear(WHITE);
        let font = create_test_font();
        renderer.draw_text(&font, 10, 10, "Hello", 16.0, BLACK);

        // The pixel buffer should have non-white pixels where text was drawn
        let has_ink = renderer.buffer.iter().any(|&p| p != WHITE.to_u32());
        assert!(has_ink, "Drawing text should modify pixels in the buffer");
    }

    #[test]
    fn test_draw_text_empty() {
        let mut renderer = Renderer::new(100, 50);
        renderer.clear(WHITE);
        let font = create_test_font();
        renderer.draw_text(&font, 10, 10, "", 16.0, BLACK);

        // Buffer should remain all white
        let all_white = renderer.buffer.iter().all(|&p| p == WHITE.to_u32());
        assert!(all_white, "Empty text should not modify the buffer");
    }

    #[test]
    fn test_draw_text_clipping() {
        let mut renderer = Renderer::new(50, 50);
        renderer.clear(WHITE);
        let font = create_test_font();
        // Draw way off-screen - should not panic
        renderer.draw_text(&font, 10000, 10000, "Off screen", 16.0, BLACK);
        let all_white = renderer.buffer.iter().all(|&p| p == WHITE.to_u32());
        assert!(all_white, "Off-screen text should not modify the buffer");
    }
}
