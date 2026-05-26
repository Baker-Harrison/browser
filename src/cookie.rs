//! Cookie storage module for the browser
//!
//! This module provides HTTP cookie management including:
//! - Parsing Set-Cookie headers
//! - Storing cookies by domain
//! - Retrieving cookies for requests
//! - Handling cookie expiration and security flags

use crate::error::{BrowserError, Result};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents an HTTP cookie
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cookie {
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// Domain the cookie belongs to
    pub domain: String,
    /// Path the cookie applies to
    pub path: String,
    /// Expiration time as Unix timestamp (None for session cookies)
    pub expires: Option<u64>,
    /// Whether the cookie should only be sent over HTTPS
    pub secure: bool,
    /// Whether the cookie is HTTP-only (not accessible via JavaScript)
    pub http_only: bool,
    /// Creation time for sorting
    pub creation_time: u64,
}

impl Cookie {
    /// Create a new cookie
    pub fn new(name: String, value: String, domain: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Cookie {
            name,
            value,
            domain,
            path: "/".to_string(),
            expires: None,
            secure: false,
            http_only: false,
            creation_time: now,
        }
    }

    /// Check if the cookie is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now >= expires
        } else {
            false
        }
    }

    /// Check if the cookie matches a domain
    pub fn matches_domain(&self, domain: &str) -> bool {
        // Exact match
        if self.domain == domain {
            return true;
        }

        // Domain cookie (starts with dot) - check if it's a subdomain
        if self.domain.starts_with('.') {
            let domain_without_dot = &self.domain[1..];
            if domain.ends_with(domain_without_dot) {
                // Ensure we're not matching "example.com" with ".com"
                let remaining = &domain[..domain.len() - domain_without_dot.len()];
                return remaining.is_empty() || remaining.ends_with('.');
            }
        }

        false
    }

    /// Check if the cookie matches a path
    pub fn matches_path(&self, path: &str) -> bool {
        if path.starts_with(&self.path) {
            // If the cookie path is a prefix, ensure it's at a path boundary
            if path.len() == self.path.len() {
                return true;
            }
            // Check if the next character is a path separator
            let next_char = path.chars().nth(self.path.len());
            next_char == Some('/') || next_char.is_none()
        } else {
            false
        }
    }

    /// Check if the cookie can be sent over the current connection
    pub fn can_send(&self, is_secure: bool) -> bool {
        // Secure cookies can only be sent over HTTPS
        if self.secure && !is_secure {
            return false;
        }
        !self.is_expired()
    }
}

/// Cookie jar for storing and retrieving cookies
#[derive(Debug, Default)]
pub struct CookieJar {
    /// Cookies indexed by domain
    cookies: HashMap<String, Vec<Cookie>>,
}

impl CookieJar {
    /// Create a new empty cookie jar
    pub fn new() -> Self {
        CookieJar {
            cookies: HashMap::new(),
        }
    }

    /// Parse a Set-Cookie header and add the cookie to the jar
    ///
    /// # Arguments
    ///
    /// * `header_value` - The value of the Set-Cookie header
    /// * `request_domain` - The domain from the request URL
    /// * `request_path` - The path from the request URL
    /// * `is_secure` - Whether the connection is HTTPS
    ///
    /// # Errors
    ///
    /// Returns `BrowserError::StorageError` if the header is malformed
    pub fn set_cookie(
        &mut self,
        header_value: &str,
        request_domain: &str,
        request_path: &str,
        is_secure: bool,
    ) -> Result<()> {
        let cookie = Self::parse_set_cookie(header_value, request_domain, request_path, is_secure)?;

        // Don't set secure cookies over non-secure connections
        if cookie.secure && !is_secure {
            return Ok(());
        }

        // Remove existing cookie with same name, domain, and path
        let domain_cookies = self.cookies.entry(cookie.domain.clone()).or_default();
        domain_cookies.retain(|c| !(c.name == cookie.name && c.path == cookie.path));

        // Add the new cookie
        domain_cookies.push(cookie);

        Ok(())
    }

    /// Get cookies to send with a request
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain of the request
    /// * `path` - The path of the request
    /// * `is_secure` - Whether the connection is HTTPS
    ///
    /// # Returns
    ///
    /// Returns a Cookie header value string
    pub fn get_cookies(&self, domain: &str, path: &str, is_secure: bool) -> String {
        let mut matching_cookies: Vec<&Cookie> = Vec::new();

        // Collect cookies from matching domains
        for cookies in self.cookies.values() {
            for cookie in cookies {
                if cookie.matches_domain(domain)
                    && cookie.matches_path(path)
                    && cookie.can_send(is_secure)
                {
                    matching_cookies.push(cookie);
                }
            }
        }

        // Sort by creation time (newer first) and path length (longer first)
        matching_cookies.sort_by(|a, b| {
            b.creation_time
                .cmp(&a.creation_time)
                .then_with(|| b.path.len().cmp(&a.path.len()))
        });

        // Build Cookie header value
        matching_cookies
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Remove expired cookies from the jar
    pub fn cleanup_expired(&mut self) {
        for cookies in self.cookies.values_mut() {
            cookies.retain(|c| !c.is_expired());
        }
    }

    /// Clear all cookies
    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    /// Clear cookies for a specific domain
    pub fn clear_domain(&mut self, domain: &str) {
        self.cookies.remove(domain);
    }

    /// Get the number of cookies stored
    pub fn len(&self) -> usize {
        self.cookies.values().map(|v| v.len()).sum()
    }

    /// Check if the jar is empty
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }

    /// Parse a Set-Cookie header value
    fn parse_set_cookie(
        header_value: &str,
        request_domain: &str,
        request_path: &str,
        _is_secure: bool,
    ) -> Result<Cookie> {
        let parts: Vec<&str> = header_value.split(';').map(|s| s.trim()).collect();

        if parts.is_empty() {
            return Err(BrowserError::StorageError(
                "Empty Set-Cookie header".to_string(),
            ));
        }

        // Parse name=value pair
        let name_value: Vec<&str> = parts[0].splitn(2, '=').collect();
        if name_value.len() != 2 {
            return Err(BrowserError::StorageError(format!(
                "Invalid cookie name=value pair: {}",
                parts[0]
            )));
        }

        let name = name_value[0].trim().to_string();
        let value = name_value[1].trim().to_string();

        // Create cookie with request domain
        let mut cookie = Cookie::new(name, value, request_domain.to_string());

        // Set default path from request
        if let Some(last_slash) = request_path.rfind('/') {
            if last_slash > 0 {
                cookie.path = request_path[..last_slash].to_string();
            }
        }

        // Parse attributes
        for part in &parts[1..] {
            let part_lower = part.to_lowercase();
            if part_lower == "secure" {
                cookie.secure = true;
            } else if part_lower == "httponly" {
                cookie.http_only = true;
            } else if part_lower.starts_with("expires=") {
                let expires_str = &part[8..];
                if let Some(timestamp) = Self::parse_expires(expires_str) {
                    cookie.expires = Some(timestamp);
                }
            } else if part_lower.starts_with("max-age=") {
                let max_age_str = &part[8..];
                if let Ok(max_age) = max_age_str.parse::<u64>() {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if max_age == 0 {
                        // Max-Age=0 means delete immediately
                        cookie.expires = Some(now);
                    } else {
                        cookie.expires = Some(now + max_age);
                    }
                }
            } else if part_lower.starts_with("domain=") {
                let domain_str = &part[7..].trim();
                if !domain_str.is_empty() {
                    cookie.domain = domain_str.to_string();
                    // Ensure domain starts with dot for domain cookies
                    if !cookie.domain.starts_with('.') && cookie.domain != request_domain {
                        cookie.domain = format!(".{}", cookie.domain);
                    }
                }
            } else if part_lower.starts_with("path=") {
                let path_str = &part[5..].trim();
                if !path_str.is_empty() {
                    cookie.path = path_str.to_string();
                }
            }
        }

        Ok(cookie)
    }

    /// Parse an Expires attribute value
    fn parse_expires(expires_str: &str) -> Option<u64> {
        // Try to parse common date formats
        // RFC 6265 format: "Wed, 09 Jun 2021 10:18:14 GMT"
        // We'll use a simple heuristic for now
        // In production, use a proper date parsing library

        // For testing purposes, accept Unix timestamp
        if let Ok(timestamp) = expires_str.parse::<u64>() {
            return Some(timestamp);
        }

        // Try to parse as RFC 1123 format
        // This is a simplified version - production code should use chrono or time crate
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_new() {
        let cookie = Cookie::new(
            "name".to_string(),
            "value".to_string(),
            "example.com".to_string(),
        );
        assert_eq!(cookie.name, "name");
        assert_eq!(cookie.value, "value");
        assert_eq!(cookie.domain, "example.com");
        assert_eq!(cookie.path, "/");
        assert!(!cookie.secure);
        assert!(!cookie.http_only);
        assert!(cookie.expires.is_none());
    }

    #[test]
    fn test_cookie_is_expired() {
        let mut cookie = Cookie::new(
            "name".to_string(),
            "value".to_string(),
            "example.com".to_string(),
        );

        // Session cookie is not expired
        assert!(!cookie.is_expired());

        // Expired cookie
        let past = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 100;
        cookie.expires = Some(past);
        assert!(cookie.is_expired());

        // Future cookie
        let future = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 1000;
        cookie.expires = Some(future);
        assert!(!cookie.is_expired());
    }

    #[test]
    fn test_cookie_matches_domain() {
        let cookie = Cookie::new(
            "name".to_string(),
            "value".to_string(),
            "example.com".to_string(),
        );

        // Exact match
        assert!(cookie.matches_domain("example.com"));

        // Different domain
        assert!(!cookie.matches_domain("other.com"));

        // Domain cookie
        let domain_cookie = Cookie::new(
            "name".to_string(),
            "value".to_string(),
            ".example.com".to_string(),
        );
        assert!(domain_cookie.matches_domain("example.com"));
        assert!(domain_cookie.matches_domain("sub.example.com"));
        assert!(!domain_cookie.matches_domain("other.com"));
    }

    #[test]
    fn test_cookie_matches_path() {
        let mut cookie = Cookie::new(
            "name".to_string(),
            "value".to_string(),
            "example.com".to_string(),
        );
        cookie.path = "/app".to_string();

        assert!(cookie.matches_path("/app"));
        assert!(cookie.matches_path("/app/"));
        assert!(cookie.matches_path("/app/page"));
        assert!(!cookie.matches_path("/other"));
        assert!(!cookie.matches_path("/application"));
    }

    #[test]
    fn test_cookie_can_send() {
        let mut cookie = Cookie::new(
            "name".to_string(),
            "value".to_string(),
            "example.com".to_string(),
        );

        // Non-secure cookie can be sent over HTTP
        assert!(cookie.can_send(false));
        assert!(cookie.can_send(true));

        // Secure cookie can only be sent over HTTPS
        cookie.secure = true;
        assert!(!cookie.can_send(false));
        assert!(cookie.can_send(true));

        // Expired cookie cannot be sent
        let past = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 100;
        cookie.expires = Some(past);
        assert!(!cookie.can_send(true));
    }

    #[test]
    fn test_cookie_jar_new() {
        let jar = CookieJar::new();
        assert!(jar.is_empty());
        assert_eq!(jar.len(), 0);
    }

    #[test]
    fn test_cookie_jar_set_cookie() {
        let mut jar = CookieJar::new();

        jar.set_cookie("name=value", "example.com", "/", false)
            .unwrap();

        assert!(!jar.is_empty());
        assert_eq!(jar.len(), 1);
    }

    #[test]
    fn test_cookie_jar_set_cookie_with_attributes() {
        let mut jar = CookieJar::new();

        jar.set_cookie(
            "name=value; Secure; HttpOnly; Path=/app",
            "example.com",
            "/",
            true,
        )
        .unwrap();

        assert_eq!(jar.len(), 1);
    }

    #[test]
    fn test_cookie_jar_set_cookie_overwrites() {
        let mut jar = CookieJar::new();

        jar.set_cookie("name=value1", "example.com", "/", false)
            .unwrap();
        jar.set_cookie("name=value2", "example.com", "/", false)
            .unwrap();

        assert_eq!(jar.len(), 1);
        let cookies = jar.get_cookies("example.com", "/", false);
        assert!(cookies.contains("name=value2"));
    }

    #[test]
    fn test_cookie_jar_get_cookies() {
        let mut jar = CookieJar::new();

        jar.set_cookie("cookie1=value1", "example.com", "/", false)
            .unwrap();
        jar.set_cookie("cookie2=value2", "example.com", "/", false)
            .unwrap();

        let cookies = jar.get_cookies("example.com", "/", false);
        assert!(cookies.contains("cookie1=value1"));
        assert!(cookies.contains("cookie2=value2"));
    }

    #[test]
    fn test_cookie_jar_get_cookies_domain_matching() {
        let mut jar = CookieJar::new();

        jar.set_cookie("name=value", ".example.com", "/", false)
            .unwrap();

        // Should match subdomain
        let cookies = jar.get_cookies("sub.example.com", "/", false);
        assert!(cookies.contains("name=value"));

        // Should match exact domain
        let cookies = jar.get_cookies("example.com", "/", false);
        assert!(cookies.contains("name=value"));

        // Should not match different domain
        let cookies = jar.get_cookies("other.com", "/", false);
        assert!(cookies.is_empty());
    }

    #[test]
    fn test_cookie_jar_get_cookies_path_matching() {
        let mut jar = CookieJar::new();

        jar.set_cookie("name=value; Path=/app", "example.com", "/", false)
            .unwrap();

        // Should match exact path
        let cookies = jar.get_cookies("example.com", "/app", false);
        assert!(cookies.contains("name=value"));

        // Should match subpath
        let cookies = jar.get_cookies("example.com", "/app/page", false);
        assert!(cookies.contains("name=value"));

        // Should not match different path
        let cookies = jar.get_cookies("example.com", "/other", false);
        assert!(!cookies.contains("name=value"));
    }

    #[test]
    fn test_cookie_jar_get_cookies_secure() {
        let mut jar = CookieJar::new();

        jar.set_cookie("secure=secret; Secure", "example.com", "/", true)
            .unwrap();

        // Should not send over HTTP
        let cookies = jar.get_cookies("example.com", "/", false);
        assert!(cookies.is_empty());

        // Should send over HTTPS
        let cookies = jar.get_cookies("example.com", "/", true);
        assert!(cookies.contains("secure=secret"));
    }

    #[test]
    fn test_cookie_jar_cleanup_expired() {
        let mut jar = CookieJar::new();

        // Add expired cookie
        let past = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 100;
        let mut cookie = Cookie::new(
            "expired".to_string(),
            "value".to_string(),
            "example.com".to_string(),
        );
        cookie.expires = Some(past);
        jar.cookies
            .entry("example.com".to_string())
            .or_default()
            .push(cookie);

        // Add valid cookie
        jar.set_cookie("valid=value", "example.com", "/", false)
            .unwrap();

        assert_eq!(jar.len(), 2);

        jar.cleanup_expired();

        assert_eq!(jar.len(), 1);
        let cookies = jar.get_cookies("example.com", "/", false);
        assert!(cookies.contains("valid=value"));
        assert!(!cookies.contains("expired=value"));
    }

    #[test]
    fn test_cookie_jar_clear() {
        let mut jar = CookieJar::new();

        jar.set_cookie("name=value", "example.com", "/", false)
            .unwrap();
        assert_eq!(jar.len(), 1);

        jar.clear();
        assert!(jar.is_empty());
    }

    #[test]
    fn test_cookie_jar_clear_domain() {
        let mut jar = CookieJar::new();

        jar.set_cookie("name1=value1", "example.com", "/", false)
            .unwrap();
        jar.set_cookie("name2=value2", "other.com", "/", false)
            .unwrap();

        assert_eq!(jar.len(), 2);

        jar.clear_domain("example.com");

        assert_eq!(jar.len(), 1);
        let cookies = jar.get_cookies("other.com", "/", false);
        assert!(cookies.contains("name2=value2"));
    }

    #[test]
    fn test_parse_set_cookie_simple() {
        let cookie = CookieJar::parse_set_cookie("name=value", "example.com", "/", false).unwrap();

        assert_eq!(cookie.name, "name");
        assert_eq!(cookie.value, "value");
        assert_eq!(cookie.domain, "example.com");
    }

    #[test]
    fn test_parse_set_cookie_with_secure() {
        let cookie =
            CookieJar::parse_set_cookie("name=value; Secure", "example.com", "/", true).unwrap();

        assert!(cookie.secure);
    }

    #[test]
    fn test_parse_set_cookie_with_httponly() {
        let cookie =
            CookieJar::parse_set_cookie("name=value; HttpOnly", "example.com", "/", false).unwrap();

        assert!(cookie.http_only);
    }

    #[test]
    fn test_parse_set_cookie_with_path() {
        let cookie =
            CookieJar::parse_set_cookie("name=value; Path=/app", "example.com", "/", false)
                .unwrap();

        assert_eq!(cookie.path, "/app");
    }

    #[test]
    fn test_parse_set_cookie_with_domain() {
        let cookie = CookieJar::parse_set_cookie(
            "name=value; Domain=example.com",
            "sub.example.com",
            "/",
            false,
        )
        .unwrap();

        assert_eq!(cookie.domain, ".example.com");
    }

    #[test]
    fn test_parse_set_cookie_with_max_age() {
        let cookie =
            CookieJar::parse_set_cookie("name=value; Max-Age=3600", "example.com", "/", false)
                .unwrap();

        assert!(cookie.expires.is_some());
        let expires = cookie.expires.unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(expires > now);
    }

    #[test]
    fn test_parse_set_cookie_max_age_zero() {
        let cookie =
            CookieJar::parse_set_cookie("name=value; Max-Age=0", "example.com", "/", false)
                .unwrap();

        assert!(cookie.expires.is_some());
        let expires = cookie.expires.unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(expires <= now);
    }

    #[test]
    fn test_parse_set_cookie_invalid() {
        let result = CookieJar::parse_set_cookie("", "example.com", "/", false);
        assert!(result.is_err());

        let result = CookieJar::parse_set_cookie("invalid", "example.com", "/", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_secure_cookie_rejected_over_http() {
        let mut jar = CookieJar::new();

        // Should not add secure cookie over HTTP
        jar.set_cookie("name=value; Secure", "example.com", "/", false)
            .unwrap();

        assert!(jar.is_empty());
    }

    #[test]
    fn test_cookie_path_default() {
        let cookie =
            CookieJar::parse_set_cookie("name=value", "example.com", "/app/page", false).unwrap();

        // Default path should be the directory of the request path
        assert_eq!(cookie.path, "/app");
    }

    #[test]
    fn test_cookie_jar_multiple_domains() {
        let mut jar = CookieJar::new();

        jar.set_cookie("name1=value1", "example.com", "/", false)
            .unwrap();
        jar.set_cookie("name2=value2", "other.com", "/", false)
            .unwrap();

        assert_eq!(jar.len(), 2);

        let cookies1 = jar.get_cookies("example.com", "/", false);
        assert!(cookies1.contains("name1=value1"));
        assert!(!cookies1.contains("name2=value2"));

        let cookies2 = jar.get_cookies("other.com", "/", false);
        assert!(!cookies2.contains("name1=value1"));
        assert!(cookies2.contains("name2=value2"));
    }
}
