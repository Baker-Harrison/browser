//! 2D rendering and composition module
//!
//! Handles software rasterization of UI elements, text, and images.
//! Implements the Compositor trait for executing display lists.

use crate::font::FontSystem;
use crate::image::{DecodedImage, ImageOptions, calculate_image_rect};
use crate::paint::{DisplayCommand, DisplayList};

/// Color represented as RGBA
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

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
pub const LIGHT_BLUE: Color = Color::new(135, 206, 250);

// Dark theme colors (Catppuccin-inspired)
#[allow(dead_code)]
pub const CHROME_BG: Color = Color::new(30, 30, 46);
#[allow(dead_code)]
pub const CHROME_SURFACE: Color = Color::new(49, 50, 68);
#[allow(dead_code)]
pub const CHROME_OVERLAY: Color = Color::new(69, 71, 90);
#[allow(dead_code)]
pub const CHROME_TEXT: Color = Color::new(205, 214, 244);
#[allow(dead_code)]
pub const CHROME_SUBTEXT: Color = Color::new(166, 173, 200);
#[allow(dead_code)]
pub const CHROME_LAVENDER: Color = Color::new(180, 190, 254);
#[allow(dead_code)]
pub const CHROME_GREEN: Color = Color::new(166, 227, 161);
#[allow(dead_code)]
pub const CHROME_BORDER: Color = Color::new(88, 91, 112);
#[allow(dead_code)]
pub const CHROME_TAB_ACTIVE: Color = Color::new(49, 50, 68);
#[allow(dead_code)]
pub const CHROME_TAB_INACTIVE: Color = Color::new(36, 36, 54);

/// Rectangle for drawing operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Check if a point is inside the rectangle
    pub fn contains(&self, px: u32, py: u32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }
}

/// Browser chrome layout constants
pub mod layout {
    use super::Rect;

    pub const CHROME_HEIGHT: u32 = 78;
    pub const TAB_BAR_HEIGHT: u32 = 38;
    pub const NAV_ROW_HEIGHT: u32 = 40;
    pub const BUTTON_SIZE: u32 = 30;
    pub const BUTTON_SPACING: u32 = 5;
    pub const BUTTON_PADDING: u32 = 6;
    pub const ADDRESS_BAR_WIDTH: u32 = 400;
    pub const ADDRESS_BAR_HEIGHT: u32 = 30;

    /// Back button rectangle
    pub fn back_button() -> Rect {
        Rect::new(
            BUTTON_SPACING,
            TAB_BAR_HEIGHT + BUTTON_PADDING,
            BUTTON_SIZE,
            BUTTON_SIZE,
        )
    }

    /// Forward button rectangle
    pub fn forward_button() -> Rect {
        Rect::new(
            BUTTON_SPACING * 2 + BUTTON_SIZE,
            TAB_BAR_HEIGHT + BUTTON_PADDING,
            BUTTON_SIZE,
            BUTTON_SIZE,
        )
    }

    /// Refresh button rectangle
    #[allow(dead_code)]
    pub fn refresh_button() -> Rect {
        Rect::new(
            BUTTON_SPACING * 3 + BUTTON_SIZE * 2,
            TAB_BAR_HEIGHT + BUTTON_PADDING,
            BUTTON_SIZE,
            BUTTON_SIZE,
        )
    }

    /// Home button rectangle
    #[allow(dead_code)]
    pub fn home_button() -> Rect {
        Rect::new(
            BUTTON_SPACING * 4 + BUTTON_SIZE * 3,
            TAB_BAR_HEIGHT + BUTTON_PADDING,
            BUTTON_SIZE,
            BUTTON_SIZE,
        )
    }

    /// Address bar rectangle (responsive to window width)
    pub fn address_bar(window_width: u32) -> Rect {
        let x = BUTTON_SPACING * 5 + BUTTON_SIZE * 4;
        let go_w = 50;
        let width = window_width
            .saturating_sub(x)
            .saturating_sub(go_w)
            .saturating_sub(BUTTON_SPACING * 2);
        Rect::new(
            x,
            TAB_BAR_HEIGHT + BUTTON_PADDING,
            width,
            NAV_ROW_HEIGHT - BUTTON_PADDING * 2,
        )
    }

    /// Go button rectangle (responsive to window width)
    pub fn go_button(window_width: u32) -> Rect {
        Rect::new(
            window_width.saturating_sub(50 + BUTTON_SPACING),
            TAB_BAR_HEIGHT + BUTTON_PADDING,
            50,
            NAV_ROW_HEIGHT - BUTTON_PADDING * 2,
        )
    }

    /// Tab bar area
    #[allow(dead_code)]
    pub fn tab_bar_area(window_width: u32) -> Rect {
        Rect::new(0, 0, window_width, TAB_BAR_HEIGHT)
    }

    /// Full chrome area
    pub fn chrome_area(window_width: u32) -> Rect {
        Rect::new(0, 0, window_width, CHROME_HEIGHT)
    }

    /// Content area (below chrome)
    pub fn content_area(window_width: u32, window_height: u32) -> Rect {
        Rect::new(
            0,
            CHROME_HEIGHT,
            window_width,
            window_height.saturating_sub(CHROME_HEIGHT),
        )
    }
}

/// Compositor trait for executing display lists into pixel buffers
pub trait Compositor {
    /// Execute a display list into the pixel buffer.
    fn composite(&mut self, commands: &DisplayList);

    /// Draw browser chrome over the content.
    fn draw_chrome(&mut self, width: u32);

    /// Return the final pixel buffer (0x00RRGGBB per pixel).
    fn buffer(&self) -> &[u32];
}

/// 2D software renderer
pub struct Renderer {
    buffer: Vec<u32>,
    width: u32,
    height: u32,
    /// Global viewport clipping rectangle (e.g. content area)
    clip_rect: Option<Rect>,
    /// Font system owned by the renderer
    font_system: Option<FontSystem>,
    /// Viewport scroll x offset for content rendering
    pub scroll_x: i32,
    /// Viewport scroll y offset for content rendering
    pub scroll_y: i32,
}

impl Renderer {
    /// Create a new renderer with the given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let buffer = vec![WHITE.to_u32(); (width * height) as usize];
        let font_system = FontSystem::new().ok();
        Renderer {
            buffer,
            width,
            height,
            clip_rect: None,
            font_system,
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    /// Set scroll offsets to offset coordinate spaces of drawing commands in composite
    pub fn set_scroll_offset(&mut self, scroll_x: i32, scroll_y: i32) {
        self.scroll_x = scroll_x;
        self.scroll_y = scroll_y;
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

    /// Get the internal font system
    pub fn font_system(&self) -> Option<&FontSystem> {
        self.font_system.as_ref()
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
        self.clip_rect = None; // Reset clip on resize
    }

    /// Set a global clipping rectangle for subsequent draw operations
    pub fn set_clip(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.clip_rect = Some(Rect::new(x, y, width, height));
    }

    /// Clear the global clipping rectangle
    pub fn clear_clip(&mut self) {
        self.clip_rect = None;
    }

    /// Check if a point is clipped by the global clip rect
    fn is_clipped(&self, x: u32, y: u32) -> bool {
        if let Some(clip) = self.clip_rect {
            x < clip.x || x >= clip.x + clip.width || y < clip.y || y >= clip.y + clip.height
        } else {
            false
        }
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
                    if self.is_clipped(x as u32, y as u32) {
                        continue;
                    }
                    let index = y * self.width as usize + x;
                    self.buffer[index] = color_u32;
                }
            }
        }
    }

    /// Draw a filled rectangle with a scroll offset
    pub fn fill_rect_offset(&mut self, rect: Rect, offset_x: i32, offset_y: i32, color: Color) {
        let color_u32 = color.to_u32();
        let x_start = (rect.x as i32 - offset_x) as usize;
        let y_start = (rect.y as i32 - offset_y) as usize;
        let width = rect.width as usize;
        let height = rect.height as usize;

        for y in y_start..(y_start + height) {
            for x in x_start..(x_start + width) {
                if y < self.height as usize && x < self.width as usize {
                    if self.is_clipped(x as u32, y as u32) {
                        continue;
                    }
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
                    if self.is_clipped(x as u32, y as u32) {
                        continue;
                    }
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
                    if self.is_clipped(x as u32, y as u32) {
                        continue;
                    }
                    let index = y * self.width as usize + x;
                    self.buffer[index] = color_u32;
                }
            }
        }

        // Left edge
        for y in rect.y as usize..(rect.y + rect.height) as usize {
            for x in rect.x as usize..(rect.x as usize + thickness) {
                if y < self.height as usize && x < self.width as usize {
                    if self.is_clipped(x as u32, y as u32) {
                        continue;
                    }
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
                    if self.is_clipped(x as u32, y as u32) {
                        continue;
                    }
                    let index = y * self.width as usize + x;
                    self.buffer[index] = color_u32;
                }
            }
        }
    }

    /// Fill a rectangle with rounded corners
    #[allow(dead_code)]
    pub fn fill_rounded_rect(&mut self, rect: Rect, radius: u32, color: Color) {
        let color_u32 = color.to_u32();
        let r = radius.min(rect.width / 2).min(rect.height / 2);

        for py in 0..rect.height {
            for px in 0..rect.width {
                let x = rect.x + px;
                let y = rect.y + py;
                if x >= self.width || y >= self.height {
                    continue;
                }
                if self.is_clipped(x, y) {
                    continue;
                }

                // Check corners
                let in_corner = |cx: u32, cy: u32| -> bool {
                    let dx = if px < cx {
                        cx - px
                    } else if px >= rect.width - cx {
                        px - (rect.width - cx - 1)
                    } else {
                        return false;
                    };
                    let dy = if py < cy {
                        cy - py
                    } else if py >= rect.height - cy {
                        py - (rect.height - cy - 1)
                    } else {
                        return false;
                    };
                    (dx * dx + dy * dy) > cx * cy
                };

                if in_corner(r, r) {
                    continue;
                }

                let index = (y * self.width + x) as usize;
                self.buffer[index] = color_u32;
            }
        }
    }

    /// Draw a 1px line using Bresenham's algorithm
    #[allow(dead_code)]
    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color) {
        let color_u32 = color.to_u32();
        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx: i32 = if x1 < x2 { 1 } else { -1 };
        let sy: i32 = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut cx = x1;
        let mut cy = y1;

        loop {
            if cx >= 0
                && cy >= 0
                && (cx as u32) < self.width
                && (cy as u32) < self.height
                && !self.is_clipped(cx as u32, cy as u32)
            {
                let index = (cy as u32 * self.width + cx as u32) as usize;
                self.buffer[index] = color_u32;
            }
            if cx == x2 && cy == y2 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                cx += sx;
            }
            if e2 <= dx {
                err += dx;
                cy += sy;
            }
        }
    }

    /// Draw the browser chrome UI with a dark two-row design
    pub fn draw_chrome_impl(&mut self, window_width: u32) {
        // === Row 1: Tab bar ===
        let tab_bar = layout::tab_bar_area(window_width);
        self.fill_rect(tab_bar, CHROME_BG);

        // Active tab
        let tab_x = 8u32;
        let tab_y = 6u32;
        let tab_w = 200u32;
        let tab_h = layout::TAB_BAR_HEIGHT - 6;
        let tab_rect = Rect::new(tab_x, tab_y, tab_w, tab_h);
        self.fill_rounded_rect(tab_rect, 6, CHROME_TAB_ACTIVE);

        // Tab title text
        self.draw_text(tab_x + 12, tab_y + 8, "Example Domain", 13.0, CHROME_TEXT);

        // Close button "x" on the tab
        let close_x = tab_x + tab_w - 22;
        let close_y = tab_y + 8;
        self.draw_text(close_x, close_y, "x", 12.0, CHROME_SUBTEXT);

        // "+" new tab button
        let new_tab_x = tab_x + tab_w + 6;
        let new_tab_rect = Rect::new(new_tab_x, tab_y + 4, 28, tab_h - 8);
        self.fill_rounded_rect(new_tab_rect, 4, CHROME_OVERLAY);
        self.draw_text(new_tab_x + 8, tab_y + 8, "+", 14.0, CHROME_SUBTEXT);

        // === Row 2: Navigation bar ===
        let nav_y = layout::TAB_BAR_HEIGHT;
        let nav_rect = Rect::new(0, nav_y, window_width, layout::NAV_ROW_HEIGHT);
        self.fill_rect(nav_rect, CHROME_BG);

        // Back button
        let back = layout::back_button();
        self.fill_rounded_rect(back, 6, CHROME_SURFACE);
        // Draw left arrow using lines
        let bcx = back.x as i32 + back.width as i32 / 2;
        let bcy = back.y as i32 + back.height as i32 / 2;
        self.draw_line(bcx + 5, bcy - 6, bcx - 5, bcy, CHROME_TEXT);
        self.draw_line(bcx - 5, bcy, bcx + 5, bcy + 6, CHROME_TEXT);

        // Forward button
        let fwd = layout::forward_button();
        self.fill_rounded_rect(fwd, 6, CHROME_SURFACE);
        // Draw right arrow using lines
        let fcx = fwd.x as i32 + fwd.width as i32 / 2;
        let fcy = fwd.y as i32 + fwd.height as i32 / 2;
        self.draw_line(fcx - 5, fcy - 6, fcx + 5, fcy, CHROME_TEXT);
        self.draw_line(fcx + 5, fcy, fcx - 5, fcy + 6, CHROME_TEXT);

        // Refresh button
        let refresh = layout::refresh_button();
        self.fill_rounded_rect(refresh, 6, CHROME_SURFACE);
        self.draw_text(refresh.x + 6, refresh.y + 6, "R", 14.0, CHROME_TEXT);

        // Home button
        let home = layout::home_button();
        self.fill_rounded_rect(home, 6, CHROME_SURFACE);
        self.draw_text(home.x + 6, home.y + 6, "H", 14.0, CHROME_TEXT);

        // Address bar
        let address_bar = layout::address_bar(window_width);
        self.fill_rounded_rect(address_bar, 6, CHROME_SURFACE);
        self.draw_rect(address_bar, CHROME_BORDER, 1);

        // Security indicator (green dot)
        let dot_x = address_bar.x + 8;
        let dot_y = address_bar.y + address_bar.height / 2 - 4;
        let dot_rect = Rect::new(dot_x, dot_y, 8, 8);
        self.fill_rounded_rect(dot_rect, 4, CHROME_GREEN);

        // Go button
        let go_button = layout::go_button(window_width);
        self.fill_rounded_rect(go_button, 6, CHROME_LAVENDER);
        self.draw_text(go_button.x + 14, go_button.y + 6, "Go", 13.0, CHROME_BG);

        // Bottom border
        self.draw_line(
            0,
            layout::CHROME_HEIGHT as i32 - 1,
            window_width as i32 - 1,
            layout::CHROME_HEIGHT as i32 - 1,
            CHROME_BORDER,
        );
    }

    /// Draw text at the given position using the font system.
    pub fn draw_text(&mut self, x: u32, y: u32, text: &str, size: f32, color: Color) {
        let font_system = self.font_system.take();
        if let Some(ref font) = font_system {
            let rasterized = font.rasterize(text, size, (color.r, color.g, color.b, color.a));
            if rasterized.width == 0 || rasterized.height == 0 {
                self.font_system = font_system;
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
                        if self.is_clipped(dest_x as u32, dest_y as u32) {
                            continue;
                        }
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
        self.font_system = font_system;
    }

    /// Fill a rectangle with clipping stack limits applied
    fn fill_rect_clipped(
        &mut self,
        rect: Rect,
        color: Color,
        clip_x: u32,
        clip_y: u32,
        clip_w: u32,
        clip_h: u32,
    ) {
        let color_u32 = color.to_u32();

        // Calculate intersection of rect and command clip rect
        let x_start = rect.x.max(clip_x);
        let y_start = rect.y.max(clip_y);
        let x_end = (rect.x + rect.width).min(clip_x + clip_w);
        let y_end = (rect.y + rect.height).min(clip_y + clip_h);

        if x_start >= x_end || y_start >= y_end {
            return; // No intersection
        }

        for y in y_start..y_end {
            for x in x_start..x_end {
                if y < self.height && x < self.width {
                    if self.is_clipped(x, y) {
                        continue;
                    }
                    let index = (y * self.width + x) as usize;
                    self.buffer[index] = color_u32;
                }
            }
        }
    }

    /// Draw a decoded image onto the buffer with scaling options and clipping stack limits
    pub fn draw_image_impl(
        &mut self,
        image: &DecodedImage,
        bounds: (u32, u32, u32, u32),
        options: &ImageOptions,
        clip_limits: Option<(u32, u32, u32, u32)>,
    ) {
        if !image.is_valid() {
            return;
        }

        let (dest_x, dest_y, dest_w, dest_h) = calculate_image_rect(image, bounds, options);

        let scale_x = dest_w as f32 / image.width as f32;
        let scale_y = dest_h as f32 / image.height as f32;

        for dy in 0..dest_h {
            for dx in 0..dest_w {
                let screen_x = dest_x + dx;
                let screen_y = dest_y + dy;

                if screen_x >= self.width || screen_y >= self.height {
                    continue;
                }

                if self.is_clipped(screen_x, screen_y) {
                    continue;
                }

                if let Some((cx, cy, cw, ch)) = clip_limits {
                    if screen_x < cx || screen_x >= cx + cw || screen_y < cy || screen_y >= cy + ch
                    {
                        continue;
                    }
                }

                let src_x = (dx as f32 / scale_x) as u32;
                let src_y = (dy as f32 / scale_y) as u32;

                if let Some((r, g, b, a)) = image.get_pixel(src_x, src_y) {
                    let screen_idx = (screen_y * self.width + screen_x) as usize;

                    if a == 255 {
                        self.buffer[screen_idx] =
                            ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                    } else if a > 0 {
                        let bg = self.buffer[screen_idx];
                        let bg_r = (bg >> 16) & 0xFF;
                        let bg_g = (bg >> 8) & 0xFF;
                        let bg_b = bg & 0xFF;

                        let a_u32 = a as u32;
                        let inv_a = 255 - a_u32;

                        let new_r = ((r as u32 * a_u32 + bg_r * inv_a) / 255) as u8;
                        let new_g = ((g as u32 * a_u32 + bg_g * inv_a) / 255) as u8;
                        let new_b = ((b as u32 * a_u32 + bg_b * inv_a) / 255) as u8;

                        self.buffer[screen_idx] =
                            ((new_r as u32) << 16) | ((new_g as u32) << 8) | (new_b as u32);
                    }
                }
            }
        }
    }
}

impl Compositor for Renderer {
    fn composite(&mut self, commands: &DisplayList) {
        let mut clip_stack: Vec<(u32, u32, u32, u32)> = Vec::new();
        // Since content area starts at CHROME_HEIGHT, we must offset everything by that vertically
        let content_y_offset = crate::renderer::layout::CHROME_HEIGHT as i32;

        for cmd in commands {
            match cmd {
                DisplayCommand::FillRect { rect, color } => {
                    // Apply scroll and chrome offset
                    let x = ((rect.x as i32 - self.scroll_x).max(0)) as u32;
                    let y = ((rect.y as i32 - self.scroll_y + content_y_offset).max(0)) as u32;
                    let width = rect.width.max(0.0) as u32;
                    let height = rect.height.max(0.0) as u32;
                    let draw_rect = Rect::new(x, y, width, height);
                    let color_rgba = Color::rgba(color.0, color.1, color.2, color.3);

                    if let Some(&(clip_x, clip_y, clip_w, clip_h)) = clip_stack.last() {
                        self.fill_rect_clipped(
                            draw_rect, color_rgba, clip_x, clip_y, clip_w, clip_h,
                        );
                    } else {
                        self.fill_rect(draw_rect, color_rgba);
                    }
                }
                DisplayCommand::DrawText {
                    x,
                    y,
                    text,
                    size,
                    color,
                } => {
                    // Apply scroll and chrome offset
                    let x_u32 = ((*x as i32 - self.scroll_x).max(0)) as u32;
                    let y_u32 = ((*y as i32 - self.scroll_y + content_y_offset).max(0)) as u32;
                    let color_rgba = Color::rgba(color.0, color.1, color.2, color.3);

                    // If clipped, we skip rendering characters that fall outside the active clip limits.
                    // For simplicity, we delegate clipping check inside draw_text's pixel loop.
                    self.draw_text(x_u32, y_u32, text, *size, color_rgba);
                }
                DisplayCommand::DrawImage { rect, data } => {
                    if let Ok(image) = crate::image::ImageLoader::load(data.as_slice()) {
                        // Apply scroll and chrome offset
                        let x = ((rect.x as i32 - self.scroll_x).max(0)) as u32;
                        let y = ((rect.y as i32 - self.scroll_y + content_y_offset).max(0)) as u32;
                        let width = rect.width.max(0.0) as u32;
                        let height = rect.height.max(0.0) as u32;
                        let bounds = (x, y, width, height);
                        let opts = ImageOptions::default();

                        let active_clip = clip_stack.last().cloned();
                        self.draw_image_impl(&image, bounds, &opts, active_clip);
                    }
                }
                DisplayCommand::ClipRect(rect) => {
                    // Apply scroll and chrome offset
                    let x = ((rect.x as i32 - self.scroll_x).max(0)) as u32;
                    let y = ((rect.y as i32 - self.scroll_y + content_y_offset).max(0)) as u32;
                    let width = rect.width.max(0.0) as u32;
                    let height = rect.height.max(0.0) as u32;
                    clip_stack.push((x, y, width, height));
                }
                DisplayCommand::PopClip => {
                    clip_stack.pop();
                }
            }
        }
    }

    fn draw_chrome(&mut self, width: u32) {
        self.draw_chrome_impl(width);
    }

    fn buffer(&self) -> &[u32] {
        &self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::LayoutRect;

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
        assert_eq!(renderer.width(), 800);
        assert_eq!(renderer.height(), 600);
        assert_eq!(renderer.buffer().len(), 800 * 600);
    }

    #[test]
    fn test_renderer_clear() {
        let mut renderer = Renderer::new(100, 100);
        renderer.clear(BLACK);
        assert_eq!(renderer.buffer()[0], BLACK.to_u32());
        assert_eq!(renderer.buffer()[9999], BLACK.to_u32());
    }

    #[test]
    fn test_layout_constants() {
        let back = layout::back_button();
        assert_eq!(back.width, layout::BUTTON_SIZE);
        assert_eq!(back.height, layout::BUTTON_SIZE);

        let forward = layout::forward_button();
        assert!(forward.x > back.x);
    }

    fn create_test_font() -> crate::font::FontSystem {
        crate::font::FontSystem::new().expect("Failed to load system font for tests")
    }

    #[test]
    fn test_renderer_draw_text() {
        let mut renderer = Renderer::new(200, 100);
        renderer.clear(WHITE);
        renderer.draw_text(10, 10, "Hello", 16.0, BLACK);

        let has_ink = renderer.buffer().iter().any(|&p| p != WHITE.to_u32());
        assert!(has_ink, "Drawing text should modify pixels in the buffer");
    }

    #[test]
    fn test_renderer_clip() {
        let mut renderer = Renderer::new(100, 100);
        renderer.clear(WHITE);
        renderer.set_clip(20, 20, 40, 40);

        let rect = Rect::new(10, 10, 80, 80);
        renderer.fill_rect(rect, BLACK);

        assert_eq!(renderer.buffer()[0], WHITE.to_u32());
        assert_eq!(renderer.buffer()[15], WHITE.to_u32());

        let clip_start = (25 * 100 + 25) as usize;
        assert_eq!(renderer.buffer()[clip_start], BLACK.to_u32());

        renderer.clear_clip();
    }

    #[test]
    fn test_fill_rect_offset() {
        let mut renderer = Renderer::new(100, 100);
        renderer.clear(WHITE);

        let rect = Rect::new(50, 50, 20, 20);
        renderer.fill_rect_offset(rect, 10, 10, BLACK);

        let index = 40 * 100 + 40;
        assert_eq!(renderer.buffer()[index], BLACK.to_u32());

        let index_content = 30 * 100 + 30;
        assert_eq!(renderer.buffer()[index_content], WHITE.to_u32());
    }

    #[test]
    fn test_draw_image() {
        let mut renderer = Renderer::new(100, 100);
        renderer.clear(WHITE);

        let mut data = vec![0u8; 400];
        for i in (0..400).step_by(4) {
            data[i] = 255;
            data[i + 3] = 255;
        }
        let image = crate::image::DecodedImage::new(10, 10, data, crate::image::ImageFormat::Png);

        let opts = crate::image::ImageOptions::default();
        renderer.draw_image_impl(&image, (10, 10, 20, 20), &opts, None);

        let has_red = renderer.buffer().iter().any(|&p| p != WHITE.to_u32());
        assert!(has_red, "Drawing image should modify pixels");
    }

    #[test]
    fn test_compositor_fill_rect() {
        let mut renderer = Renderer::new(100, 100);
        renderer.clear(WHITE);

        let commands = vec![DisplayCommand::FillRect {
            rect: LayoutRect::new(10.0, 10.0, 50.0, 50.0),
            color: (255, 0, 0, 255),
        }];

        renderer.composite(&commands);

        let idx = (15 * 100 + 15) as usize;
        let pixel = renderer.buffer()[idx];
        let red = (pixel >> 16) & 0xFF;
        assert_eq!(red, 255);
    }

    #[test]
    fn test_compositor_clip_rect() {
        let mut renderer = Renderer::new(100, 100);
        renderer.clear(WHITE);

        let commands = vec![
            DisplayCommand::ClipRect(LayoutRect::new(20.0, 20.0, 30.0, 30.0)),
            DisplayCommand::FillRect {
                rect: LayoutRect::new(10.0, 10.0, 50.0, 50.0),
                color: (255, 0, 0, 255),
            },
            DisplayCommand::PopClip,
        ];

        renderer.composite(&commands);

        let outside_idx = (15 * 100 + 15) as usize;
        assert_eq!(renderer.buffer()[outside_idx], WHITE.to_u32());

        let inside_idx = (25 * 100 + 25) as usize;
        let pixel = renderer.buffer()[inside_idx];
        let red = (pixel >> 16) & 0xFF;
        assert_eq!(red, 255);
    }
}
