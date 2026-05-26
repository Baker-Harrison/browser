//! URL handling module for the browser
//!
//! This module provides URL validation, parsing, and scheme checking
//! to ensure URLs are properly formatted before attempting to load them.

use crate::error::{BrowserError, Result};
use std::fmt;

/// A validated, parsed URL.
///
/// This trait defines the interface for URL representations in the browser.
/// All URL implementations must provide access to the standard URL components.
pub trait ParsedUrl {
    /// Get the URL scheme (e.g., "http" or "https")
    fn scheme(&self) -> &str;

    /// Get the host (domain or IP address)
    fn host(&self) -> &str;

    /// Get the port if specified, otherwise None
    fn port(&self) -> Option<u16>;

    /// Get the path component of the URL
    fn path(&self) -> &str;

    /// Get the query string if present
    fn query(&self) -> Option<&str>;

    /// Get the fragment (anchor) if present
    fn fragment(&self) -> Option<&str>;

    /// Check if the URL uses a secure scheme (e.g., HTTPS)
    fn is_secure(&self) -> bool;

    /// Get the URL as a string
    fn as_str(&self) -> &str;
}

/// URL parser trait for creating ParsedUrl instances from strings.
///
/// This trait defines the interface for URL parsing implementations.
/// Each parser type can produce its own ParsedUrl implementation.
pub trait UrlParser {
    /// The type of ParsedUrl this parser produces
    type Output: ParsedUrl;

    /// Parse a URL string into a ParsedUrl
    ///
    /// # Arguments
    ///
    /// * `input` - The URL string to parse
    ///
    /// # Returns
    ///
    /// Returns a parsed URL if the input is valid
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is malformed or uses an unsupported scheme
    fn parse(input: &str) -> Result<Self::Output>;
}

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
        if let Some(port) = self.parsed.port() {
            Some(port)
        } else {
            let host = self.parsed.host_str().unwrap_or("");
            if !host.is_empty() {
                let host_idx = self.original.find(host);
                if let Some(idx) = host_idx {
                    let after_host = &self.original[idx + host.len()..];
                    if let Some(stripped) = after_host.strip_prefix(':') {
                        let port_part = stripped
                            .chars()
                            .take_while(|c| c.is_ascii_digit())
                            .collect::<String>();
                        if let Ok(port) = port_part.parse::<u16>() {
                            return Some(port);
                        }
                    }
                }
            }
            None
        }
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

impl ParsedUrl for BrowserUrl {
    fn scheme(&self) -> &str {
        self.parsed.scheme()
    }

    fn host(&self) -> &str {
        self.parsed.host_str().unwrap_or("")
    }

    fn port(&self) -> Option<u16> {
        self.port()
    }

    fn path(&self) -> &str {
        self.parsed.path()
    }

    fn query(&self) -> Option<&str> {
        self.parsed.query()
    }

    fn fragment(&self) -> Option<&str> {
        self.parsed.fragment()
    }

    fn is_secure(&self) -> bool {
        self.scheme() == "https"
    }

    fn as_str(&self) -> &str {
        &self.original
    }
}

/// URL parser implementation for BrowserUrl
pub struct BrowserUrlParser;

impl UrlParser for BrowserUrlParser {
    type Output = BrowserUrl;

    fn parse(input: &str) -> Result<Self::Output> {
        BrowserUrl::parse(input)
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

    // Tests for ParsedUrl trait
    #[test]
    fn test_parsed_url_trait_scheme() {
        let url: Box<dyn ParsedUrl> = Box::new(BrowserUrl::parse("https://example.com").unwrap());
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn test_parsed_url_trait_host() {
        let url: Box<dyn ParsedUrl> = Box::new(BrowserUrl::parse("https://example.com").unwrap());
        assert_eq!(url.host(), "example.com");
    }

    #[test]
    fn test_parsed_url_trait_port() {
        let url: Box<dyn ParsedUrl> =
            Box::new(BrowserUrl::parse("https://example.com:8080").unwrap());
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn test_parsed_url_trait_port_none() {
        let url: Box<dyn ParsedUrl> = Box::new(BrowserUrl::parse("https://example.com").unwrap());
        assert_eq!(url.port(), None);
    }

    #[test]
    fn test_parsed_url_trait_path() {
        let url: Box<dyn ParsedUrl> =
            Box::new(BrowserUrl::parse("https://example.com/path/to/page").unwrap());
        assert_eq!(url.path(), "/path/to/page");
    }

    #[test]
    fn test_parsed_url_trait_query() {
        let url: Box<dyn ParsedUrl> =
            Box::new(BrowserUrl::parse("https://example.com?query=test").unwrap());
        assert_eq!(url.query(), Some("query=test"));
    }

    #[test]
    fn test_parsed_url_trait_query_none() {
        let url: Box<dyn ParsedUrl> = Box::new(BrowserUrl::parse("https://example.com").unwrap());
        assert_eq!(url.query(), None);
    }

    #[test]
    fn test_parsed_url_trait_fragment() {
        let url: Box<dyn ParsedUrl> =
            Box::new(BrowserUrl::parse("https://example.com#section").unwrap());
        assert_eq!(url.fragment(), Some("section"));
    }

    #[test]
    fn test_parsed_url_trait_fragment_none() {
        let url: Box<dyn ParsedUrl> = Box::new(BrowserUrl::parse("https://example.com").unwrap());
        assert_eq!(url.fragment(), None);
    }

    #[test]
    fn test_parsed_url_trait_is_secure() {
        let url: Box<dyn ParsedUrl> = Box::new(BrowserUrl::parse("https://example.com").unwrap());
        assert!(url.is_secure());
    }

    #[test]
    fn test_parsed_url_trait_is_not_secure() {
        let url: Box<dyn ParsedUrl> = Box::new(BrowserUrl::parse("http://example.com").unwrap());
        assert!(!url.is_secure());
    }

    #[test]
    fn test_parsed_url_trait_as_str() {
        let url_str = "https://example.com/path";
        let url: Box<dyn ParsedUrl> = Box::new(BrowserUrl::parse(url_str).unwrap());
        assert_eq!(url.as_str(), url_str);
    }

    // Tests for UrlParser trait
    #[test]
    fn test_url_parser_trait_parse() {
        let url = BrowserUrlParser::parse("https://example.com").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "example.com");
    }

    #[test]
    fn test_url_parser_trait_parse_with_path() {
        let url = BrowserUrlParser::parse("https://example.com/path/to/page").unwrap();
        assert_eq!(url.path(), "/path/to/page");
    }

    #[test]
    fn test_url_parser_trait_parse_with_query() {
        let url = BrowserUrlParser::parse("https://example.com?query=test").unwrap();
        assert_eq!(url.query(), Some("query=test"));
    }

    #[test]
    fn test_url_parser_trait_parse_with_fragment() {
        let url = BrowserUrlParser::parse("https://example.com#section").unwrap();
        assert_eq!(url.fragment(), Some("section"));
    }

    #[test]
    fn test_url_parser_trait_parse_with_port() {
        let url = BrowserUrlParser::parse("https://example.com:8080").unwrap();
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn test_url_parser_trait_parse_invalid() {
        let result = BrowserUrlParser::parse("not a url");
        assert!(result.is_err());
    }

    #[test]
    fn test_url_parser_trait_parse_unsupported_scheme() {
        let result = BrowserUrlParser::parse("ftp://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_url_parser_trait_parse_without_host() {
        let result = BrowserUrlParser::parse("https://");
        assert!(result.is_err());
    }

    // Test that BrowserUrl implements both traits
    #[test]
    fn test_browser_url_implements_both_traits() {
        let url = BrowserUrl::parse("https://example.com").unwrap();

        // Test ParsedUrl trait methods
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "example.com");
        assert!(url.is_secure());

        // Test UrlParser trait via BrowserUrlParser
        let url2 = BrowserUrlParser::parse("https://example.com").unwrap();
        assert_eq!(url2.scheme(), "https");
        assert_eq!(url2.host(), "example.com");
    }

    // Test complex URL with all components
    #[test]
    fn test_complex_url_all_components() {
        let url =
            BrowserUrlParser::parse("https://example.com:8443/path/to/page?key=value#section")
                .unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.port(), Some(8443));
        assert_eq!(url.path(), "/path/to/page");
        assert_eq!(url.query(), Some("key=value"));
        assert_eq!(url.fragment(), Some("section"));
        assert!(url.is_secure());
    }

    // Test HTTP with default port
    #[test]
    fn test_http_default_port() {
        let url = BrowserUrlParser::parse("http://example.com:80").unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.port(), Some(80));
        assert!(!url.is_secure());
    }

    // Test HTTPS with default port
    #[test]
    fn test_https_default_port() {
        let url = BrowserUrlParser::parse("https://example.com:443").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.port(), Some(443));
        assert!(url.is_secure());
    }

    // Test URL with empty path
    #[test]
    fn test_url_empty_path() {
        let url = BrowserUrlParser::parse("https://example.com").unwrap();
        assert_eq!(url.path(), "/");
    }

    // Test URL with multiple query parameters
    #[test]
    fn test_url_multiple_query_params() {
        let url = BrowserUrlParser::parse("https://example.com?key1=value1&key2=value2").unwrap();
        assert_eq!(url.query(), Some("key1=value1&key2=value2"));
    }
}
