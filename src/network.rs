//! Network stack module for the browser
//!
//! This module provides HTTP client functionality for fetching web content,
//! including timeout support and error handling.

use crate::error::{BrowserError, Result};
use std::time::Duration;

/// Default timeout for HTTP requests (30 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Raw HTTP response with status, headers, body, and metadata
pub struct RawResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub url: String,
    pub mime_type: String,
}

/// Trait for fetching resources over HTTP
#[allow(dead_code)]
pub trait Fetcher {
    fn fetch(&self, url: &str) -> Result<RawResponse>;
    fn post(&self, url: &str, body: &[u8], content_type: &str) -> Result<RawResponse>;
}

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

    /// Perform a GET request and return the full raw response
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch
    ///
    /// # Returns
    ///
    /// Returns a `RawResponse` containing status, headers, body, and metadata
    ///
    /// # Errors
    ///
    /// Returns `BrowserError::NetworkError` if the request fails
    /// Returns `BrowserError::TimeoutError` if the request times out
    /// Returns `BrowserError::ConnectionError` if connection fails
    pub fn fetch(&self, url: &str) -> Result<RawResponse> {
        let response = self.client.get(url).send().map_err(|e| {
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

        let status = response.status().as_u16();
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
            .collect();

        let mime_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .split(';')
            .next()
            .unwrap_or("application/octet-stream")
            .trim()
            .to_string();

        let url = response.url().to_string();
        let body = response.bytes().map_err(|e| {
            BrowserError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(RawResponse {
            status,
            headers,
            body: body.to_vec(),
            url,
            mime_type,
        })
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

impl Fetcher for HttpClient {
    fn fetch(&self, url: &str) -> Result<RawResponse> {
        self.fetch(url)
    }

    fn post(&self, url: &str, body: &[u8], content_type: &str) -> Result<RawResponse> {
        let response = self
            .client
            .post(url)
            .body(body.to_vec())
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    BrowserError::TimeoutError(format!(
                        "POST to '{}' timed out after {:?}",
                        url, self.timeout
                    ))
                } else if e.is_connect() {
                    BrowserError::ConnectionError(format!("Failed to connect to '{}': {}", url, e))
                } else {
                    BrowserError::NetworkError(format!("Failed to POST to '{}': {}", url, e))
                }
            })?;

        let status = response.status().as_u16();
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
            .collect();

        let mime_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .split(';')
            .next()
            .unwrap_or("application/octet-stream")
            .trim()
            .to_string();

        let url = response.url().to_string();
        let body = response.bytes().map_err(|e| {
            BrowserError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(RawResponse {
            status,
            headers,
            body: body.to_vec(),
            url,
            mime_type,
        })
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

    #[test]
    fn test_raw_response_structure() {
        let response = RawResponse {
            status: 200,
            headers: vec![("Content-Type".to_string(), "text/html".to_string())],
            body: b"<html></html>".to_vec(),
            url: "https://example.com".to_string(),
            mime_type: "text/html".to_string(),
        };
        assert_eq!(response.status, 200);
        assert_eq!(response.mime_type, "text/html");
        assert_eq!(response.body, b"<html></html>");
        assert_eq!(response.url, "https://example.com");
        assert_eq!(response.headers.len(), 1);
        assert_eq!(response.headers[0].0, "Content-Type");
    }

    #[test]
    fn test_fetch_invalid_url() {
        let client = HttpClient::new().unwrap();
        let result = client.fetch("not a valid url");
        assert!(result.is_err());
    }

    #[test]
    fn test_fetcher_trait_impl() {
        fn takes_fetcher(_f: &impl Fetcher) {
            // Verify that HttpClient implements Fetcher
        }
        let client = HttpClient::new().unwrap();
        takes_fetcher(&client);
    }

    #[test]
    #[ignore] // This test makes a real network request - ignore by default
    fn test_fetch_returns_raw() {
        let client = HttpClient::new().unwrap();
        let result = client.fetch("https://example.com");
        assert!(result.is_ok());
        let raw = result.unwrap();
        assert_eq!(raw.status, 200);
        assert!(!raw.body.is_empty());
        assert_eq!(raw.mime_type, "text/html");
        assert!(raw.url.contains("example.com"));
    }
}
