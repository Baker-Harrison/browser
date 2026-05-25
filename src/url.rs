//! URL handling module for the browser
//!
//! This module provides URL validation, parsing, and scheme checking
//! to ensure URLs are properly formatted before attempting to load them.

use crate::error::{BrowserError, Result};
use std::fmt;

/// Supported URL schemes
const SUPPORTED_SCHEMES: &[&str] = &["http", "https"];

/// Represents a validated and parsed URL
#[derive(Debug, Clone)]
pub struct BrowserUrl {
    /// The original URL string
    original: String,
    /// The parsed URL
    parsed: url::Url,
}

impl BrowserUrl {
    /// Parse and validate a URL string
    ///
    /// # Arguments
    ///
    /// * `url_str` - The URL string to parse and validate
    ///
    /// # Returns
    ///
    /// Returns a `BrowserUrl` if the URL is valid and uses a supported scheme
    ///
    /// # Errors
    ///
    /// Returns `BrowserError::InvalidUrl` if the URL is malformed
    /// Returns `BrowserError::UnsupportedScheme` if the scheme is not http or https
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::url::BrowserUrl;
    ///
    /// let url = BrowserUrl::parse("https://example.com").unwrap();
    /// assert_eq!(url.scheme(), "https");
    /// ```
    pub fn parse(url_str: &str) -> Result<Self> {
        // Parse the URL using the url crate
        let parsed = url::Url::parse(url_str).map_err(|e| {
            BrowserError::UrlParseError(format!("Failed to parse URL '{}': {}", url_str, e))
        })?;

        // Validate the scheme
        let scheme = parsed.scheme();
        if !SUPPORTED_SCHEMES.contains(&scheme) {
            return Err(BrowserError::UnsupportedScheme(format!(
                "Scheme '{}' is not supported. Supported schemes: {}",
                scheme,
                SUPPORTED_SCHEMES.join(", ")
            )));
        }

        // Ensure the URL has a host
        if parsed.host().is_none() {
            return Err(BrowserError::InvalidUrl(format!(
                "URL '{}' does not have a valid host",
                url_str
            )));
        }

        Ok(BrowserUrl {
            original: url_str.to_string(),
            parsed,
        })
    }

    /// Get the URL scheme (e.g., "http" or "https")
    pub fn scheme(&self) -> &str {
        self.parsed.scheme()
    }

    /// Get the host (domain or IP address)
    pub fn host(&self) -> &str {
        self.parsed.host_str().unwrap_or("")
    }

    /// Get the port if specified, otherwise None
    pub fn port(&self) -> Option<u16> {
        self.parsed.port()
    }

    /// Get the path component of the URL
    pub fn path(&self) -> &str {
        self.parsed.path()
    }

    /// Get the query string if present
    pub fn query(&self) -> Option<&str> {
        self.parsed.query()
    }

    /// Get the fragment (anchor) if present
    pub fn fragment(&self) -> Option<&str> {
        self.parsed.fragment()
    }

    /// Get the original URL string
    pub fn as_str(&self) -> &str {
        &self.original
    }

    /// Check if the URL uses HTTPS
    pub fn is_secure(&self) -> bool {
        self.scheme() == "https"
    }

    /// Convert to the underlying url::Url
    pub fn as_url(&self) -> &url::Url {
        &self.parsed
    }
}

impl fmt::Display for BrowserUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.original)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_https_url() {
        let url = BrowserUrl::parse("https://example.com").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "example.com");
        assert!(url.is_secure());
    }

    #[test]
    fn test_parse_valid_http_url() {
        let url = BrowserUrl::parse("http://example.com").unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host(), "example.com");
        assert!(!url.is_secure());
    }

    #[test]
    fn test_parse_url_with_path() {
        let url = BrowserUrl::parse("https://example.com/path/to/page").unwrap();
        assert_eq!(url.path(), "/path/to/page");
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = BrowserUrl::parse("https://example.com?query=test").unwrap();
        assert_eq!(url.query(), Some("query=test"));
    }

    #[test]
    fn test_parse_url_with_fragment() {
        let url = BrowserUrl::parse("https://example.com#section").unwrap();
        assert_eq!(url.fragment(), Some("section"));
    }

    #[test]
    fn test_parse_url_with_port() {
        let url = BrowserUrl::parse("https://example.com:8080").unwrap();
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn test_parse_invalid_url() {
        let result = BrowserUrl::parse("not a url");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unsupported_scheme() {
        let result = BrowserUrl::parse("ftp://example.com");
        assert!(result.is_err());
        match result {
            Err(BrowserError::UnsupportedScheme(msg)) => {
                assert!(msg.contains("ftp"));
            }
            _ => panic!("Expected UnsupportedScheme error"),
        }
    }

    #[test]
    fn test_parse_url_without_host() {
        let result = BrowserUrl::parse("https://");
        assert!(result.is_err());
    }

    #[test]
    fn test_display() {
        let url = BrowserUrl::parse("https://example.com").unwrap();
        assert_eq!(url.to_string(), "https://example.com");
    }

    #[test]
    fn test_as_str() {
        let url_str = "https://example.com/path";
        let url = BrowserUrl::parse(url_str).unwrap();
        assert_eq!(url.as_str(), url_str);
    }
}
