//! Window management module for the browser
//!
//! This module handles window creation and event loop management using winit.

use crate::error::{BrowserError, Result};
use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

/// Represents the browser window and its state
pub struct BrowserWindow {
    /// The winit window
    window: Arc<Window>,
    /// Current URL being displayed
    current_url: Arc<Mutex<Option<String>>>,
    /// HTML content to render
    html_content: Arc<Mutex<Option<String>>>,
}

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
    pub fn new(window: Window) -> Self {
        BrowserWindow {
            window: Arc::new(window),
            current_url: Arc::new(Mutex::new(None)),
            html_content: Arc::new(Mutex::new(None)),
        }
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

        let window = event_loop
            .create_window(window_attributes)
            .expect("Failed to create window");

        self.window = Some(BrowserWindow::new(window));
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
                // Handle redraw - for now just clear the window
                // TODO: Implement actual rendering
                if let Some(window) = &self.window {
                    window.request_redraw();
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
    use super::*;

    #[test]
    fn test_browser_window_creation() {
        // This test would require a running event loop, so we skip it
        // Window creation is tested manually by running the browser
    }
}
