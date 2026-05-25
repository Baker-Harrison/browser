use thiserror::Error;

/// Main error type for the browser application
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum BrowserError {
    /// URL-related errors
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("URL scheme not supported: {0}")]
    UnsupportedScheme(String),

    #[error("Failed to parse URL: {0}")]
    UrlParseError(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Failed to load configuration file: {0}")]
    ConfigLoadError(String),

    #[error("Invalid configuration value: {0}")]
    InvalidConfigValue(String),

    /// Network errors
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Connection failed: {0}")]
    ConnectionError(String),

    #[error("Timeout while connecting to: {0}")]
    TimeoutError(String),

    /// Rendering errors
    #[error("Rendering error: {0}")]
    RenderingError(String),

    #[error("Failed to parse HTML: {0}")]
    HtmlParseError(String),

    #[error("CSS parsing error: {0}")]
    CssParseError(String),

    /// JavaScript errors
    #[error("JavaScript error: {0}")]
    JavaScriptError(String),

    #[error("JavaScript execution failed: {0}")]
    JavaScriptExecutionError(String),

    /// Storage errors
    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Failed to read from storage: {0}")]
    StorageReadError(String),

    #[error("Failed to write to storage: {0}")]
    StorageWriteError(String),

    /// IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic errors
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// Result type alias for browser operations
pub type Result<T> = std::result::Result<T, BrowserError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BrowserError::InvalidUrl("not a url".to_string());
        assert_eq!(err.to_string(), "Invalid URL: not a url");
    }

    #[test]
    fn test_error_debug() {
        let err = BrowserError::InvalidUrl("not a url".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InvalidUrl"));
    }

    #[test]
    fn test_result_type() {
        let result: Result<()> = Ok(());
        assert!(result.is_ok());

        let result: Result<()> = Err(BrowserError::InternalError("test".to_string()));
        assert!(result.is_err());
    }
}
