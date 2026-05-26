//! Window management module for the browser
//!
//! This module handles window creation and event loop management using winit
//! and softbuffer for 2D rendering. It integrates viewport scrolling,
//! click navigation, and event dispatch.

use crate::error::{BrowserError, Result};
use crate::history::History;
use crate::html::HtmlDocument;
use crate::network::HttpClient;
use crate::renderer::{Compositor, Renderer, WHITE};
use crate::url::BrowserUrl;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

/// Represents the scrollable viewport inside the browser window
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub scroll_x: i32,
    pub scroll_y: i32,
    pub content_width: u32,
    pub content_height: u32,
}

impl Viewport {
    pub fn new(width: u32, height: u32) -> Self {
        Viewport {
            width,
            height,
            scroll_x: 0,
            scroll_y: 0,
            content_width: width,
            content_height: height,
        }
    }

    pub fn set_content_size(&mut self, width: u32, height: u32) {
        self.content_width = width;
        self.content_height = height;
        self.clamp_scroll();
    }

    pub fn max_scroll_x(&self) -> i32 {
        (self.content_width as i32 - self.width as i32).max(0)
    }

    pub fn max_scroll_y(&self) -> i32 {
        (self.content_height as i32 - self.height as i32).max(0)
    }

    pub fn scroll(&mut self, dx: i32, dy: i32) {
        self.scroll_to(self.scroll_x + dx, self.scroll_y + dy);
    }

    pub fn scroll_to(&mut self, x: i32, y: i32) {
        self.scroll_x = x.clamp(0, self.max_scroll_x());
        self.scroll_y = y.clamp(0, self.max_scroll_y());
    }

    pub fn clamp_scroll(&mut self) {
        self.scroll_to(self.scroll_x, self.scroll_y);
    }

    pub fn can_scroll_x(&self) -> bool {
        self.content_width > self.width
    }

    pub fn can_scroll_y(&self) -> bool {
        self.content_height > self.height
    }

    pub fn visible_rect(&self) -> (i32, i32, u32, u32) {
        let x = self.scroll_x;
        let y = self.scroll_y;
        let w = self.width.min(self.content_width.saturating_sub(x as u32));
        let h = self
            .height
            .min(self.content_height.saturating_sub(y as u32));
        (x, y, w, h)
    }
}

/// Represents a clickable link on the page
#[derive(Debug, Clone)]
pub struct Link {
    pub url: String,
    pub rect: (u32, u32, u32, u32), // content-relative: x, y, width, height
}

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
    /// Browser navigation history
    history: Arc<Mutex<History>>,
    /// HTTP client for fetching content
    http_client: HttpClient,
    /// Parsed HTML document
    document: Arc<Mutex<Option<HtmlDocument>>>,
    /// Clickable links on the current page
    links: Arc<Mutex<Vec<Link>>>,
    /// Viewport for scrolling content
    viewport: Arc<Mutex<Viewport>>,
    /// Painted display commands list for rendering the content
    display_list: Arc<Mutex<crate::paint::DisplayList>>,
}

#[allow(dead_code)]
impl BrowserWindow {
    /// Create a new browser window
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
        let http_client = HttpClient::new()?;

        Ok(BrowserWindow {
            window: window_arc,
            surface,
            renderer,
            current_url: Arc::new(Mutex::new(None)),
            html_content: Arc::new(Mutex::new(None)),
            history: Arc::new(Mutex::new(History::new())),
            http_client,
            document: Arc::new(Mutex::new(None)),
            links: Arc::new(Mutex::new(Vec::new())),
            viewport: Arc::new(Mutex::new(Viewport::new(width, height))),
            display_list: Arc::new(Mutex::new(Vec::new())),
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

    /// Scroll the viewport by the given delta
    pub fn scroll(&self, dx: i32, dy: i32) {
        let mut viewport = self.viewport.lock().unwrap();
        viewport.scroll(dx, dy);
    }

    /// Navigate to a URL
    pub fn navigate(&self, url_str: &str) -> Result<()> {
        let url = BrowserUrl::parse(url_str)?;
        let response = self.http_client.fetch(url.as_str())?;

        let html = String::from_utf8(response.body).map_err(|e| {
            BrowserError::HtmlParseError(format!("Failed to decode response as UTF-8: {}", e))
        })?;

        let document = HtmlDocument::parse(&html)?;

        // CSS Parsing & Stylesheets Extraction
        let mut stylesheets = Vec::new();
        // Extract default styles if any or style elements
        let style_nodes = document.dom().query_selector_all("style");
        for node in style_nodes {
            for child in &node.children {
                if let crate::html::NodeKind::Text(content) = &child.kind {
                    if let Ok(sheet) = crate::css::parse(content) {
                        stylesheets.push(sheet);
                    }
                }
            }
        }

        // Apply Styles
        use crate::paint::Painter;
        let style_engine = crate::layout::StyleEngine;
        let styled_tree = style_engine.style(&document.dom().root, &stylesheets);

        // Layout (assuming layout is relative to viewport content width, e.g. 800)
        let layout_engine = crate::layout::LayoutEngine;
        let layout_tree = layout_engine.layout(&styled_tree, 800.0);

        // Paint
        let painter = crate::paint::DefaultPainter;
        let display_list = painter.paint(&layout_tree);

        // Extract links dynamically from the layout tree boxes
        let mut links = Vec::new();
        fn collect_layout_links(box_node: &crate::layout::LayoutBox, links: &mut Vec<Link>) {
            if let Some(ref href) = box_node.is_link {
                links.push(Link {
                    url: href.clone(),
                    rect: (
                        box_node.rect.x as u32,
                        box_node.rect.y as u32,
                        box_node.rect.width as u32,
                        box_node.rect.height as u32,
                    ),
                });
            }
            for child in &box_node.children {
                collect_layout_links(child, links);
            }
        }
        collect_layout_links(&layout_tree, &mut links);

        *self.current_url.lock().unwrap() = Some(url_str.to_string());
        *self.html_content.lock().unwrap() = Some(html.clone());
        *self.document.lock().unwrap() = Some(document);
        *self.links.lock().unwrap() = links;
        *self.display_list.lock().unwrap() = display_list;

        // Set title and add to history
        let title = self.extract_title(&html);
        let mut history = self.history.lock().unwrap();
        let should_push = match history.current() {
            Ok(curr) => curr != url_str,
            Err(_) => true,
        };
        if should_push {
            history.push(url_str, title)?;
        }

        // Adjust scrollable content height dynamically based on layout tree height
        let content_height = layout_tree.rect.height as u32;
        let estimated_height = content_height.max(600);
        self.viewport
            .lock()
            .unwrap()
            .set_content_size(800, estimated_height);

        Ok(())
    }

    /// Extract links from the HTML content (legacy, kept for backward compatibility if needed)
    fn extract_links(&self, _document: &HtmlDocument) -> Vec<Link> {
        Vec::new()
    }

    /// Extract title from HTML content
    fn extract_title(&self, html: &str) -> Option<String> {
        html.find("<title>").and_then(|start| {
            html[start + 7..]
                .find("</title>")
                .map(|end| html[start + 7..start + 7 + end].to_string())
        })
    }

    /// Handle a mouse click at the given screen coordinates
    pub fn handle_click(&self, screen_x: u32, screen_y: u32) -> Result<bool> {
        let content_y_offset = crate::renderer::layout::CHROME_HEIGHT;
        let width = self.window.inner_size().width;

        if screen_y < content_y_offset {
            // Click is inside browser chrome area - handle back/forward navigation
            let back_rect = crate::renderer::layout::back_button();
            let forward_rect = crate::renderer::layout::forward_button();
            let refresh_rect = crate::renderer::layout::refresh_button();
            let home_rect = crate::renderer::layout::home_button();
            let go_rect = crate::renderer::layout::go_button(width);

            if back_rect.contains(screen_x, screen_y) {
                self.go_back()?;
                return Ok(true);
            } else if forward_rect.contains(screen_x, screen_y) {
                self.go_forward()?;
                return Ok(true);
            } else if refresh_rect.contains(screen_x, screen_y) {
                if let Some(ref url) = self.get_url() {
                    self.navigate(url)?;
                }
                return Ok(true);
            } else if home_rect.contains(screen_x, screen_y) {
                self.navigate("https://example.com")?;
                return Ok(true);
            } else if go_rect.contains(screen_x, screen_y) {
                // If go is clicked, trigger a refresh or do nothing for now
                if let Some(ref url) = self.get_url() {
                    self.navigate(url)?;
                }
                return Ok(true);
            }
            return Ok(false);
        }

        // Click is inside viewport content area - translate to content layout space
        let viewport = self.viewport.lock().unwrap();
        let content_x = (screen_x as i32 + viewport.scroll_x) as u32;
        let content_y = (screen_y as i32 - content_y_offset as i32 + viewport.scroll_y) as u32;
        drop(viewport);

        let links = self.links.lock().unwrap();
        for link in links.iter() {
            let (lx, ly, lw, lh) = link.rect;
            if content_x >= lx && content_x < lx + lw && content_y >= ly && content_y < ly + lh {
                let url = link.url.clone();
                drop(links);
                let resolved_url = self.resolve_url(&url)?;
                self.navigate(&resolved_url)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Resolve a relative URL against the current base URL
    fn resolve_url(&self, url: &str) -> Result<String> {
        if url.starts_with("http://") || url.starts_with("https://") {
            return Ok(url.to_string());
        }

        let current_url = self.current_url.lock().unwrap();
        if let Some(base) = current_url.as_ref() {
            let base_url = url::Url::parse(base).map_err(|e| {
                BrowserError::UrlParseError(format!("Failed to parse base URL: {}", e))
            })?;

            let resolved = base_url.join(url).map_err(|e| {
                BrowserError::UrlParseError(format!("Failed to resolve URL: {}", e))
            })?;

            Ok(resolved.to_string())
        } else {
            Ok(url.to_string())
        }
    }

    /// Go back in history
    pub fn go_back(&self) -> Result<Option<String>> {
        let mut history = self.history.lock().unwrap();
        if history.can_go_back() {
            let url = history.back()?;
            drop(history);
            self.navigate(&url)?;
            Ok(Some(url))
        } else {
            Ok(None)
        }
    }

    /// Go forward in history
    pub fn go_forward(&self) -> Result<Option<String>> {
        let mut history = self.history.lock().unwrap();
        if history.can_go_forward() {
            let url = history.forward()?;
            drop(history);
            self.navigate(&url)?;
            Ok(Some(url))
        } else {
            Ok(None)
        }
    }

    /// Check if back navigation is possible
    pub fn can_go_back(&self) -> bool {
        self.history.lock().unwrap().can_go_back()
    }

    /// Check if forward navigation is possible
    pub fn can_go_forward(&self) -> bool {
        self.history.lock().unwrap().can_go_forward()
    }

    /// Request a redraw of the window
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// Render the browser UI and page content
    pub fn render(&mut self) {
        let size = self.window.inner_size();
        let width = size.width;
        let height = size.height;

        if width == 0 || height == 0 {
            return;
        }

        if self.renderer.width() != width || self.renderer.height() != height {
            self.renderer.resize(width, height);
            let mut viewport = self.viewport.lock().unwrap();
            viewport.width = width;
            viewport.height = height;
            viewport.clamp_scroll();
        }

        self.renderer.clear(WHITE);

        // Render viewport content (with global content clip)
        let content_area = crate::renderer::layout::content_area(width, height);
        self.renderer.set_clip(
            content_area.x,
            content_area.y,
            content_area.width,
            content_area.height,
        );

        let viewport = self.viewport.lock().unwrap();
        let (scroll_x, scroll_y) = (viewport.scroll_x, viewport.scroll_y);
        drop(viewport);

        let display_list = self.display_list.lock().unwrap().clone();
        // Composite display list
        // Apply viewport scroll offsets
        self.renderer.set_scroll_offset(scroll_x, scroll_y);
        self.renderer.composite(&display_list);
        self.renderer.set_scroll_offset(0, 0);

        // Clear content clipping to allow chrome to render over everything
        self.renderer.clear_clip();
        self.renderer.draw_chrome(width);

        // Render current URL in address bar (using CHROME_TEXT color)
        if let Some(ref url) = self.get_url() {
            let address_rect = crate::renderer::layout::address_bar(width);
            self.renderer.draw_text(
                address_rect.x + 28, // padding for padlock icon
                address_rect.y + 8,
                url,
                13.0,
                crate::renderer::CHROME_TEXT,
            );
        }

        // Present to the screen
        let width_nz = NonZeroU32::new(width).expect("width should not be 0");
        let height_nz = NonZeroU32::new(height).expect("height should not be 0");
        if let Err(e) = self.surface.resize(width_nz, height_nz) {
            eprintln!("Failed to resize surface: {}", e);
            return;
        }

        match self.surface.buffer_mut() {
            Ok(mut buffer) => {
                buffer.copy_from_slice(self.renderer.buffer());
                if let Err(e) = buffer.present() {
                    eprintln!("Failed to present buffer: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to get surface buffer: {}", e);
            }
        }
    }
}

/// Application state for the winit event handler
struct BrowserApplication {
    window: Option<BrowserWindow>,
    cursor_position: Option<(u32, u32)>,
}

impl ApplicationHandler for BrowserApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
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
                // Populate with some default content
                let _ = browser_window.navigate("https://example.com");

                self.window = Some(browser_window);
                self.cursor_position = None;
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
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = &mut self.window {
                    window.render();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(window) = &self.window {
                    let size = window.window.inner_size();
                    let x = position.x as u32;
                    let y = position.y as u32;
                    if x < size.width && y < size.height {
                        self.cursor_position = Some((x, y));
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let (Some(window), ElementState::Pressed, MouseButton::Left) =
                    (&self.window, state, button)
                {
                    if let Some((x, y)) = self.cursor_position {
                        if let Err(e) = window.handle_click(x, y) {
                            eprintln!("Failed to handle click: {}", e);
                        } else {
                            window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(window) = &self.window {
                    let (_delta_x, delta_y) = match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => {
                            (x as i32 * 40, -(y as i32 * 40))
                        }
                        winit::event::MouseScrollDelta::PixelDelta(pos) => {
                            (pos.x as i32, -pos.y as i32)
                        }
                    };
                    window.scroll(0, delta_y); // Vertical scroll only
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let (Some(window), true) = (&self.window, event.state.is_pressed()) {
                    use winit::keyboard::{Key, NamedKey};
                    let scroll_amount = match event.logical_key {
                        Key::Named(NamedKey::ArrowUp) => Some((0, -30)),
                        Key::Named(NamedKey::ArrowDown) => Some((0, 30)),
                        Key::Named(NamedKey::PageUp) => {
                            let viewport = window.viewport.lock().unwrap();
                            Some((0, -(viewport.height as i32)))
                        }
                        Key::Named(NamedKey::PageDown) => {
                            let viewport = window.viewport.lock().unwrap();
                            Some((0, viewport.height as i32))
                        }
                        Key::Named(NamedKey::Home) => {
                            let viewport = window.viewport.lock().unwrap();
                            Some((0, -viewport.scroll_y))
                        }
                        Key::Named(NamedKey::End) => {
                            let viewport = window.viewport.lock().unwrap();
                            Some((0, viewport.max_scroll_y() - viewport.scroll_y))
                        }
                        _ => None,
                    };

                    if let Some((dx, dy)) = scroll_amount {
                        window.scroll(dx, dy);
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}

/// Create and run the browser window
pub fn run_browser_window() -> Result<()> {
    let event_loop = EventLoop::new()
        .map_err(|e| BrowserError::InternalError(format!("Failed to create event loop: {}", e)))?;

    let mut app = BrowserApplication {
        window: None,
        cursor_position: None,
    };

    event_loop
        .run_app(&mut app)
        .map_err(|e| BrowserError::InternalError(format!("Event loop error: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_creation() {
        let viewport = Viewport::new(800, 600);
        assert_eq!(viewport.width, 800);
        assert_eq!(viewport.height, 600);
        assert_eq!(viewport.scroll_x, 0);
        assert_eq!(viewport.scroll_y, 0);
        assert_eq!(viewport.content_width, 800);
        assert_eq!(viewport.content_height, 600);
    }

    #[test]
    fn test_viewport_set_content_size() {
        let mut viewport = Viewport::new(800, 600);
        viewport.set_content_size(1200, 2000);
        assert_eq!(viewport.content_width, 1200);
        assert_eq!(viewport.content_height, 2000);
    }

    #[test]
    fn test_viewport_max_scroll() {
        let mut viewport = Viewport::new(800, 600);
        viewport.set_content_size(1200, 2000);
        assert_eq!(viewport.max_scroll_x(), 400);
        assert_eq!(viewport.max_scroll_y(), 1400);
    }

    #[test]
    fn test_viewport_scroll() {
        let mut viewport = Viewport::new(800, 600);
        viewport.set_content_size(1200, 2000);
        viewport.scroll(100, 200);
        assert_eq!(viewport.scroll_x, 100);
        assert_eq!(viewport.scroll_y, 200);
    }

    #[test]
    fn test_viewport_scroll_clamp() {
        let mut viewport = Viewport::new(800, 600);
        viewport.set_content_size(1200, 2000);
        viewport.scroll(1000, 3000);
        assert_eq!(viewport.scroll_x, 400);
        assert_eq!(viewport.scroll_y, 1400);
    }

    #[test]
    fn test_viewport_scroll_negative() {
        let mut viewport = Viewport::new(800, 600);
        viewport.set_content_size(1200, 2000);
        viewport.scroll(100, 200);
        viewport.scroll(-200, -300);
        assert_eq!(viewport.scroll_x, 0);
        assert_eq!(viewport.scroll_y, 0);
    }

    #[test]
    fn test_viewport_can_scroll() {
        let mut viewport = Viewport::new(800, 600);
        assert!(!viewport.can_scroll_x());
        assert!(!viewport.can_scroll_y());

        viewport.set_content_size(1200, 2000);
        assert!(viewport.can_scroll_x());
        assert!(viewport.can_scroll_y());
    }

    #[test]
    fn test_viewport_visible_rect() {
        let mut viewport = Viewport::new(800, 600);
        viewport.set_content_size(1200, 2000);
        viewport.scroll(100, 200);
        let (x, y, width, height) = viewport.visible_rect();
        assert_eq!(x, 100);
        assert_eq!(y, 200);
        assert_eq!(width, 800);
        assert_eq!(height, 600);
    }

    #[test]
    fn test_link_creation() {
        let link = Link {
            url: "https://example.com".to_string(),
            rect: (10, 20, 100, 30),
        };
        assert_eq!(link.url, "https://example.com");
        assert_eq!(link.rect, (10, 20, 100, 30));
    }

    #[test]
    fn test_resolve_url_relative() {
        // Simple manual check of resolve logic matching browser window helper
        let base = "https://example.com/dir/page.html";
        let relative = "/other/page.html";

        let base_url = url::Url::parse(base).unwrap();
        let resolved = base_url.join(relative).unwrap();
        assert_eq!(resolved.as_str(), "https://example.com/other/page.html");
    }
}
