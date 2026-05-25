//! HTML parsing module for the browser
//!
//! This module provides HTML parsing functionality to extract structure,
//! title, links, and basic content from HTML documents.

use crate::error::Result;
use scraper::{Html, Selector};

/// Represents a parsed HTML document
pub struct HtmlDocument {
    /// The parsed HTML document
    document: Html,
    /// The original HTML string
    original: String,
}

impl HtmlDocument {
    /// Parse an HTML string into a structured document
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML string to parse
    ///
    /// # Returns
    ///
    /// Returns a parsed `HtmlDocument`
    ///
    /// # Errors
    ///
    /// Returns `BrowserError::HtmlParseError` if parsing fails
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::html::HtmlDocument;
    ///
    /// let html = "<html><head><title>Test</title></head><body>Hello</body></html>";
    /// let doc = HtmlDocument::parse(html).unwrap();
    /// assert_eq!(doc.title(), Some("Test".to_string()));
    /// ```
    pub fn parse(html: &str) -> Result<Self> {
        let document = Html::parse_document(html);
        Ok(HtmlDocument {
            document,
            original: html.to_string(),
        })
    }

    /// Extract the title from the HTML document
    ///
    /// # Returns
    ///
    /// Returns the title text if found, otherwise None
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::html::HtmlDocument;
    ///
    /// let html = "<html><head><title>My Page</title></head></html>";
    /// let doc = HtmlDocument::parse(html).unwrap();
    /// assert_eq!(doc.title(), Some("My Page".to_string()));
    /// ```
    pub fn title(&self) -> Option<String> {
        let selector = Selector::parse("title").ok()?;
        let element = self.document.select(&selector).next()?;
        Some(element.text().collect::<String>())
    }

    /// Extract all links (href attributes) from the HTML document
    ///
    /// # Returns
    ///
    /// Returns a vector of link URLs found in the document
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::html::HtmlDocument;
    ///
    /// let html = r#"<html><body>
    ///     <a href="https://example.com">Link 1</a>
    ///     <a href="/about">Link 2</a>
    /// </body></html>"#;
    /// let doc = HtmlDocument::parse(html).unwrap();
    /// let links = doc.links();
    /// assert!(links.contains(&"https://example.com".to_string()));
    /// ```
    pub fn links(&self) -> Vec<String> {
        let selector = match Selector::parse("a[href]") {
            Ok(sel) => sel,
            Err(_) => return Vec::new(),
        };

        self.document
            .select(&selector)
            .filter_map(|element| element.value().attr("href"))
            .map(|href| href.to_string())
            .collect()
    }

    /// Extract all image sources (src attributes) from the HTML document
    ///
    /// # Returns
    ///
    /// Returns a vector of image URLs found in the document
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::html::HtmlDocument;
    ///
    /// let html = r#"<html><body>
    ///     <img src="image1.jpg" />
    ///     <img src="/images/image2.png" />
    /// </body></html>"#;
    /// let doc = HtmlDocument::parse(html).unwrap();
    /// let images = doc.images();
    /// assert!(images.contains(&"image1.jpg".to_string()));
    /// ```
    pub fn images(&self) -> Vec<String> {
        let selector = match Selector::parse("img[src]") {
            Ok(sel) => sel,
            Err(_) => return Vec::new(),
        };

        self.document
            .select(&selector)
            .filter_map(|element| element.value().attr("src"))
            .map(|src| src.to_string())
            .collect()
    }

    /// Extract the text content from the HTML document
    ///
    /// This returns the visible text content, stripping HTML tags
    ///
    /// # Returns
    ///
    /// Returns the text content of the document
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::html::HtmlDocument;
    ///
    /// let html = "<html><body><p>Hello <strong>World</strong></p></body></html>";
    /// let doc = HtmlDocument::parse(html).unwrap();
    /// let text = doc.text_content();
    /// assert!(text.contains("Hello"));
    /// assert!(text.contains("World"));
    /// ```
    pub fn text_content(&self) -> String {
        self.document.root_element().text().collect::<String>()
    }

    /// Extract all headings (h1-h6) from the HTML document
    ///
    /// # Returns
    ///
    /// Returns a vector of (level, text) tuples for each heading
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::html::HtmlDocument;
    ///
    /// let html = r#"<html><body>
    ///     <h1>Main Title</h1>
    ///     <h2>Subtitle</h2>
    /// </body></html>"#;
    /// let doc = HtmlDocument::parse(html).unwrap();
    /// let headings = doc.headings();
    /// assert!(headings.contains(&(1, "Main Title".to_string())));
    /// ```
    pub fn headings(&self) -> Vec<(u8, String)> {
        // Use a single selector to preserve DOM order rather than iterating
        // level by level, which would reorder headings by tag type instead of
        // their actual position in the document.
        let selector = match Selector::parse("h1, h2, h3, h4, h5, h6") {
            Ok(sel) => sel,
            Err(_) => return Vec::new(),
        };

        self.document
            .select(&selector)
            .filter_map(|element| {
                let level: u8 = element
                    .value()
                    .name()
                    .trim_start_matches('h')
                    .parse()
                    .ok()?;
                let text: String = element.text().collect();
                Some((level, text))
            })
            .collect()
    }

    /// Get the original HTML string
    pub fn as_html(&self) -> &str {
        &self.original
    }

    /// Get the length of the original HTML
    pub fn len(&self) -> usize {
        self.original.len()
    }

    /// Check if the document is empty
    pub fn is_empty(&self) -> bool {
        self.original.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_html() {
        let html = "<html><body>Hello</body></html>";
        let doc = HtmlDocument::parse(html);
        assert!(doc.is_ok());
    }

    #[test]
    fn test_parse_empty_html() {
        let html = "";
        let doc = HtmlDocument::parse(html);
        assert!(doc.is_ok());
        assert!(doc.unwrap().is_empty());
    }

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Test Page</title></head><body></body></html>";
        let doc = HtmlDocument::parse(html).unwrap();
        assert_eq!(doc.title(), Some("Test Page".to_string()));
    }

    #[test]
    fn test_extract_title_none() {
        let html = "<html><head></head><body></body></html>";
        let doc = HtmlDocument::parse(html).unwrap();
        assert_eq!(doc.title(), None);
    }

    #[test]
    fn test_extract_links() {
        let html = r#"
            <html><body>
                <a href="https://example.com">Example</a>
                <a href="/about">About</a>
                <a>No href</a>
            </body></html>
        "#;
        let doc = HtmlDocument::parse(html).unwrap();
        let links = doc.links();
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"https://example.com".to_string()));
        assert!(links.contains(&"/about".to_string()));
    }

    #[test]
    fn test_extract_images() {
        let html = r#"
            <html><body>
                <img src="image1.jpg" />
                <img src="/images/image2.png" />
                <img>No src</img>
            </body></html>
        "#;
        let doc = HtmlDocument::parse(html).unwrap();
        let images = doc.images();
        assert_eq!(images.len(), 2);
        assert!(images.contains(&"image1.jpg".to_string()));
        assert!(images.contains(&"/images/image2.png".to_string()));
    }

    #[test]
    fn test_extract_text_content() {
        let html = "<html><body><p>Hello <strong>World</strong></p></body></html>";
        let doc = HtmlDocument::parse(html).unwrap();
        let text = doc.text_content();
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn test_extract_headings() {
        let html = r#"
            <html><body>
                <h1>Main Title</h1>
                <h2>Subtitle</h2>
                <h3>Section</h3>
            </body></html>
        "#;
        let doc = HtmlDocument::parse(html).unwrap();
        let headings = doc.headings();
        assert_eq!(headings.len(), 3);
        assert!(headings.contains(&(1, "Main Title".to_string())));
        assert!(headings.contains(&(2, "Subtitle".to_string())));
        assert!(headings.contains(&(3, "Section".to_string())));
    }

    #[test]
    fn test_as_html() {
        let html = "<html><body>Test</body></html>";
        let doc = HtmlDocument::parse(html).unwrap();
        assert_eq!(doc.as_html(), html);
    }

    #[test]
    fn test_len() {
        let html = "<html><body>Test</body></html>";
        let doc = HtmlDocument::parse(html).unwrap();
        assert_eq!(doc.len(), html.len());
    }

    #[test]
    fn test_is_empty() {
        let doc = HtmlDocument::parse("").unwrap();
        assert!(doc.is_empty());

        let doc = HtmlDocument::parse("<html></html>").unwrap();
        assert!(!doc.is_empty());
    }
}
