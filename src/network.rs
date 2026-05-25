//! Network stack module for the browser
//!
//! This module provides HTTP client functionality for fetching web content,
//! including timeout support and error handling.

use crate::error::{BrowserError, Result};
use std::time::Duration;

/// Default timeout for HTTP requests (30 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// HTTP client for making web requests
pub struct HttpClient {
    /// The underlying reqwest client
    client: reqwest::blocking::Client,
    /// Request timeout
    timeout: Duration,
}

impl HttpClient {
    /// Create a new HTTP client with default settings
    ///
    /// # Returns
    ///
    /// Returns a new `HttpClient` with default timeout (30 seconds)
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::network::HttpClient;
    ///
    /// let client = HttpClient::new();
    /// ```
    pub fn new() -> Result<Self> {
        Self::with_timeout(DEFAULT_TIMEOUT)
    }

    /// Create a new HTTP client with a custom timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration for requests
    ///
    /// # Returns
    ///
    /// Returns a new `HttpClient` with the specified timeout
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::network::HttpClient;
    /// use std::time::Duration;
    ///
    /// let client = HttpClient::with_timeout(Duration::from_secs(60)).unwrap();
    /// ```
    pub fn with_timeout(timeout: Duration) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .user_agent("Browser/0.1.0")
            .build()
            .map_err(|e| {
                BrowserError::NetworkError(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(HttpClient { client, timeout })
    }

    /// Perform a GET request to the specified URL
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch
    ///
    /// # Returns
    ///
    /// Returns the response body as a string
    ///
    /// # Errors
    ///
    /// Returns `BrowserError::NetworkError` if the request fails
    /// Returns `BrowserError::TimeoutError` if the request times out
    /// Returns `BrowserError::ConnectionError` if connection fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use browser::network::HttpClient;
    ///
    /// let client = HttpClient::new().unwrap();
    /// let html = client.get("https://example.com").unwrap();
    /// println!("Received {} bytes", html.len());
    /// ```
    pub fn get(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().map_err(|e: reqwest::Error| {
            if e.is_timeout() {
                BrowserError::TimeoutError(format!(
                    "Request to '{}' timed out after {:?}",
                    url, self.timeout
                ))
            } else if e.is_connect() {
                BrowserError::ConnectionError(format!("Failed to connect to '{}': {}", url, e))
            } else {
                BrowserError::NetworkError(format!("Failed to fetch '{}': {}", url, e))
            }
        })?;

        // Check for HTTP errors
        if !response.status().is_success() {
            return Err(BrowserError::NetworkError(format!(
                "HTTP error {} for URL '{}'",
                response.status(),
                url
            )));
        }

        // Read the response body
        let body = response.text().map_err(|e| {
            BrowserError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(body)
    }

    /// Get the current timeout duration
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Set a new timeout duration
    ///
    /// Note: This creates a new client with the new timeout
    #[allow(dead_code)]
    pub fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
        self.client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .user_agent("Browser/0.1.0")
            .build()
            .map_err(|e| {
                BrowserError::NetworkError(format!("Failed to create HTTP client: {}", e))
            })?;
        self.timeout = timeout;
        Ok(())
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = HttpClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_custom_timeout() {
        let client = HttpClient::with_timeout(Duration::from_secs(60));
        assert!(client.is_ok());
        assert_eq!(client.unwrap().timeout(), Duration::from_secs(60));
    }

    #[test]
    fn test_default_client() {
        let client = HttpClient::default();
        assert_eq!(client.timeout(), DEFAULT_TIMEOUT);
    }

    #[test]
    fn test_set_timeout() {
        let mut client = HttpClient::new().unwrap();
        client.set_timeout(Duration::from_secs(45)).unwrap();
        assert_eq!(client.timeout(), Duration::from_secs(45));
    }

    #[test]
    fn test_get_invalid_url() {
        let client = HttpClient::new().unwrap();
        let result = client.get("not a valid url");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_nonexistent_domain() {
        let client = HttpClient::new().unwrap();
        let result = client.get("http://this-domain-definitely-does-not-exist-12345.com");
        assert!(result.is_err());
    }

    #[test]
    #[ignore] // This test makes a real network request - ignore by default
    fn test_get_real_website() {
        let client = HttpClient::new().unwrap();
        let result = client.get("https://example.com");
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("Example Domain"));
    }
}
