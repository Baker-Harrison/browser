//! Window management module for the browser
//!
//! This module handles window creation and event loop management using winit
//! and softbuffer for 2D rendering.

use crate::error::{BrowserError, Result};
use crate::renderer::{Renderer, WHITE};
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

/// Represents the browser window and its state
#[allow(dead_code)]
pub struct BrowserWindow {
    /// The winit window
    window: Arc<Window>,
    /// The softbuffer surface for rendering
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    /// The renderer for 2D drawing
    renderer: Renderer,
    /// Current URL being displayed
    current_url: Arc<Mutex<Option<String>>>,
    /// HTML content to render
    html_content: Arc<Mutex<Option<String>>>,
}

#[allow(dead_code)]
impl BrowserWindow {
    /// Create a new browser window
    ///
    /// # Arguments
    ///
    /// * `window` - The winit window
    ///
    /// # Returns
    ///
    /// Returns a new `BrowserWindow`
    ///
    /// # Errors
    ///
    /// Returns `BrowserError` if surface creation fails
    pub fn new(window: Window) -> Result<Self> {
        let window_arc = Arc::new(window);
        let context = softbuffer::Context::new(window_arc.clone()).map_err(|e| {
            BrowserError::InternalError(format!("Failed to create softbuffer context: {}", e))
        })?;

        let surface = softbuffer::Surface::new(&context, window_arc.clone()).map_err(|e| {
            BrowserError::InternalError(format!("Failed to create softbuffer surface: {}", e))
        })?;

        let size = window_arc.inner_size();
        let width = size.width;
        let height = size.height;

        let renderer = Renderer::new(width, height);

        Ok(BrowserWindow {
            window: window_arc,
            surface,
            renderer,
            current_url: Arc::new(Mutex::new(None)),
            html_content: Arc::new(Mutex::new(None)),
        })
    }

    /// Get the window
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Set the current URL
    pub fn set_url(&self, url: String) {
        *self.current_url.lock().unwrap() = Some(url);
    }

    /// Get the current URL
    pub fn get_url(&self) -> Option<String> {
        self.current_url.lock().unwrap().clone()
    }

    /// Set the HTML content to render
    pub fn set_html(&self, html: String) {
        *self.html_content.lock().unwrap() = Some(html);
    }

    /// Get the HTML content
    pub fn get_html(&self) -> Option<String> {
        self.html_content.lock().unwrap().clone()
    }

    /// Request a redraw of the window
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// Render the browser UI
    pub fn render(&mut self) {
        let size = self.window.inner_size();
        let width = size.width;
        let height = size.height;

        // Guard against zero-size windows to avoid panic
        if width == 0 || height == 0 {
            return;
        }

        // Resize renderer if window size changed (reuses buffer when possible)
        if self.renderer.width() != width || self.renderer.height() != height {
            self.renderer.resize(width, height);
        }

        // Clear the entire window with white
        self.renderer.clear(WHITE);

        // Draw browser chrome
        self.renderer.draw_chrome(width);

        // Present the frame
        let width_nz = NonZeroU32::new(width).expect("width should not be 0");
        let height_nz = NonZeroU32::new(height).expect("height should not be 0");
        if let Err(e) = self.surface.resize(width_nz, height_nz) {
            eprintln!("Failed to resize surface: {}", e);
            return;
        }

        match self.surface.buffer_mut() {
            Ok(mut buffer) => {
                buffer.copy_from_slice(self.renderer.buffer());
            }
            Err(e) => {
                eprintln!("Failed to get surface buffer: {}", e);
            }
        }
    }
}

/// Application state for the browser
struct BrowserApplication {
    window: Option<BrowserWindow>,
}

impl ApplicationHandler for BrowserApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window when the application resumes
        let window_attributes = WindowAttributes::default()
            .with_title("Browser")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => window,
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        match BrowserWindow::new(window) {
            Ok(browser_window) => {
                self.window = Some(browser_window);
                // Trigger initial redraw
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            Err(e) => {
                eprintln!("Failed to initialize browser window: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                // Handle window resize
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                // Handle redraw - render browser chrome
                if let Some(window) = &mut self.window {
                    window.render();
                }
            }
            _ => {}
        }
    }
}

/// Create and run the browser window
///
/// # Returns
///
/// Returns `Ok(())` if the window closed successfully
///
/// # Errors
///
/// Returns `BrowserError` if window creation or event loop fails
pub fn run_browser_window() -> Result<()> {
    let event_loop = EventLoop::new()
        .map_err(|e| BrowserError::InternalError(format!("Failed to create event loop: {}", e)))?;

    let mut app = BrowserApplication { window: None };

    event_loop
        .run_app(&mut app)
        .map_err(|e| BrowserError::InternalError(format!("Event loop error: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_browser_window_creation() {
        // This test would require a running event loop, so we skip it
        // Window creation is tested manually by running the browser
    }
}
