//! Browser History API module
//!
//! This module provides navigation history management for the browser,
//! including back/forward navigation, history stack management, and
//! JavaScript-compatible history API (history.length, history.back(), history.forward()).

use crate::error::{BrowserError, Result};
use std::fmt;

/// Maximum number of history entries to keep
const MAX_HISTORY_SIZE: usize = 1000;

/// A single entry in the browser history
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// The URL of this history entry
    url: String,
    /// Title of the page (if available)
    title: Option<String>,
    /// Timestamp when this entry was added
    timestamp: std::time::SystemTime,
}

impl HistoryEntry {
    /// Create a new history entry
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the page
    /// * `title` - Optional title of the page
    ///
    /// # Returns
    ///
    /// Returns a new `HistoryEntry`
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::history::HistoryEntry;
    ///
    /// let entry = HistoryEntry::new("https://example.com", Some("Example".to_string()));
    /// assert_eq!(entry.url(), "https://example.com");
    /// ```
    pub fn new(url: String, title: Option<String>) -> Self {
        HistoryEntry {
            url,
            title,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Get the URL of this entry
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the title of this entry
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Get the timestamp of this entry
    pub fn timestamp(&self) -> std::time::SystemTime {
        self.timestamp
    }
}

/// Browser history manager
///
/// Maintains a stack of visited URLs and supports back/forward navigation.
/// This implements the JavaScript History API semantics.
#[derive(Debug)]
pub struct History {
    /// The history stack (all visited pages)
    entries: Vec<HistoryEntry>,
    /// Current position in the history stack
    current_index: usize,
}

impl History {
    /// Create a new empty history
    ///
    /// # Returns
    ///
    /// Returns a new `History` with no entries
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::history::History;
    ///
    /// let history = History::new();
    /// assert_eq!(history.length(), 0);
    /// ```
    pub fn new() -> Self {
        History {
            entries: Vec::new(),
            current_index: 0,
        }
    }

    /// Add a new entry to the history
    ///
    /// This is called when navigating to a new URL. Any entries after the
    /// current position are discarded (following standard browser behavior).
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to add to history
    /// * `title` - Optional title of the page
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success
    ///
    /// # Errors
    ///
    /// Returns `BrowserError::StorageError` if the history is full
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::history::History;
    ///
    /// let mut history = History::new();
    /// history.push("https://example.com", Some("Example".to_string())).unwrap();
    /// assert_eq!(history.length(), 1);
    /// ```
    pub fn push(&mut self, url: &str, title: Option<String>) -> Result<()> {
        // Check if we're at capacity
        if self.entries.len() >= MAX_HISTORY_SIZE {
            return Err(BrowserError::StorageError(
                "History is full (maximum 1000 entries)".to_string(),
            ));
        }

        // Discard any forward history (entries after current position)
        if self.current_index < self.entries.len() {
            self.entries.truncate(self.current_index);
        }

        // Add the new entry
        self.entries.push(HistoryEntry::new(url.to_string(), title));
        self.current_index = self.entries.len();

        Ok(())
    }

    /// Navigate back in history
    ///
    /// # Returns
    ///
    /// Returns the URL of the previous page if available
    ///
    /// # Errors
    ///
    /// Returns `BrowserError::StorageError` if there's no previous entry
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::history::History;
    ///
    /// let mut history = History::new();
    /// history.push("https://example.com/page1", None).unwrap();
    /// history.push("https://example.com/page2", None).unwrap();
    ///
    /// let url = history.back().unwrap();
    /// assert_eq!(url, "https://example.com/page1");
    /// ```
    pub fn back(&mut self) -> Result<String> {
        if self.current_index <= 1 {
            return Err(BrowserError::StorageError(
                "No previous entry in history".to_string(),
            ));
        }

        self.current_index -= 1;
        Ok(self.entries[self.current_index - 1].url().to_string())
    }

    /// Navigate forward in history
    ///
    /// # Returns
    ///
    /// Returns the URL of the next page if available
    ///
    /// # Errors
    ///
    /// Returns `BrowserError::StorageError` if there's no next entry
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::history::History;
    ///
    /// let mut history = History::new();
    /// history.push("https://example.com/page1", None).unwrap();
    /// history.push("https://example.com/page2", None).unwrap();
    /// history.back().unwrap();
    ///
    /// let url = history.forward().unwrap();
    /// assert_eq!(url, "https://example.com/page2");
    /// ```
    pub fn forward(&mut self) -> Result<String> {
        if self.current_index >= self.entries.len() {
            return Err(BrowserError::StorageError(
                "No next entry in history".to_string(),
            ));
        }

        self.current_index += 1;
        Ok(self.entries[self.current_index - 1].url().to_string())
    }

    /// Get the current URL in history
    ///
    /// # Returns
    ///
    /// Returns the current URL if available
    ///
    /// # Errors
    ///
    /// Returns `BrowserError::StorageError` if history is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::history::History;
    ///
    /// let mut history = History::new();
    /// history.push("https://example.com", None).unwrap();
    ///
    /// let url = history.current().unwrap();
    /// assert_eq!(url, "https://example.com");
    /// ```
    pub fn current(&self) -> Result<String> {
        if self.current_index == 0 || self.entries.is_empty() {
            return Err(BrowserError::StorageError("History is empty".to_string()));
        }

        Ok(self.entries[self.current_index - 1].url().to_string())
    }

    /// Get the length of the history (JavaScript-compatible)
    ///
    /// # Returns
    ///
    /// Returns the number of entries in the history
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::history::History;
    ///
    /// let mut history = History::new();
    /// assert_eq!(history.length(), 0);
    ///
    /// history.push("https://example.com", None).unwrap();
    /// assert_eq!(history.length(), 1);
    /// ```
    pub fn length(&self) -> usize {
        self.entries.len()
    }

    /// Check if we can go back
    ///
    /// # Returns
    ///
    /// Returns true if there's a previous entry
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::history::History;
    ///
    /// let mut history = History::new();
    /// history.push("https://example.com/page1", None).unwrap();
    /// history.push("https://example.com/page2", None).unwrap();
    ///
    /// assert!(history.can_go_back());
    /// ```
    pub fn can_go_back(&self) -> bool {
        self.current_index > 1
    }

    /// Check if we can go forward
    ///
    /// # Returns
    ///
    /// Returns true if there's a next entry
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::history::History;
    ///
    /// let mut history = History::new();
    /// history.push("https://example.com/page1", None).unwrap();
    /// history.push("https://example.com/page2", None).unwrap();
    /// history.back().unwrap();
    ///
    /// assert!(history.can_go_forward());
    /// ```
    pub fn can_go_forward(&self) -> bool {
        self.current_index < self.entries.len()
    }

    /// Get all history entries
    ///
    /// # Returns
    ///
    /// Returns a slice of all history entries
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::history::History;
    ///
    /// let mut history = History::new();
    /// history.push("https://example.com/page1", None).unwrap();
    /// history.push("https://example.com/page2", None).unwrap();
    ///
    /// let entries = history.entries();
    /// assert_eq!(entries.len(), 2);
    /// ```
    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Clear all history entries
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::history::History;
    ///
    /// let mut history = History::new();
    /// history.push("https://example.com", None).unwrap();
    /// history.clear();
    ///
    /// assert_eq!(history.length(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_index = 0;
    }

    /// Get the current position in history
    ///
    /// # Returns
    ///
    /// Returns the current index (0-based)
    ///
    /// # Examples
    ///
    /// ```
    /// use browser::history::History;
    ///
    /// let mut history = History::new();
    /// history.push("https://example.com/page1", None).unwrap();
    /// history.push("https://example.com/page2", None).unwrap();
    ///
    /// assert_eq!(history.current_index(), 2);
    /// ```
    pub fn current_index(&self) -> usize {
        self.current_index
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for History {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "History ({} entries):", self.length())?;
        for (i, entry) in self.entries.iter().enumerate() {
            let marker = if i + 1 == self.current_index {
                ">"
            } else {
                " "
            };
            writeln!(
                f,
                "  {} [{}] {} - {}",
                marker,
                i + 1,
                entry.url(),
                entry.title().unwrap_or("(no title)")
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_new() {
        let history = History::new();
        assert_eq!(history.length(), 0);
        assert_eq!(history.current_index(), 0);
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_history_default() {
        let history = History::default();
        assert_eq!(history.length(), 0);
    }

    #[test]
    fn test_push_single_entry() {
        let mut history = History::new();
        let result = history.push("https://example.com", Some("Example".to_string()));
        assert!(result.is_ok());
        assert_eq!(history.length(), 1);
        assert_eq!(history.current_index(), 1);
    }

    #[test]
    fn test_push_multiple_entries() {
        let mut history = History::new();
        history.push("https://example.com/page1", None).unwrap();
        history.push("https://example.com/page2", None).unwrap();
        history.push("https://example.com/page3", None).unwrap();

        assert_eq!(history.length(), 3);
        assert_eq!(history.current_index(), 3);
    }

    #[test]
    fn test_push_discards_forward_history() {
        let mut history = History::new();
        history.push("https://example.com/page1", None).unwrap();
        history.push("https://example.com/page2", None).unwrap();
        history.push("https://example.com/page3", None).unwrap();

        // Go back
        history.back().unwrap();
        assert_eq!(history.current_index(), 2);

        // Add new entry - should discard page3
        history.push("https://example.com/page4", None).unwrap();
        assert_eq!(history.length(), 3);
        assert_eq!(history.current_index(), 3);

        // Verify page3 is gone
        let entries = history.entries();
        assert_eq!(entries[0].url(), "https://example.com/page1");
        assert_eq!(entries[1].url(), "https://example.com/page2");
        assert_eq!(entries[2].url(), "https://example.com/page4");
    }

    #[test]
    fn test_back_navigation() {
        let mut history = History::new();
        history.push("https://example.com/page1", None).unwrap();
        history.push("https://example.com/page2", None).unwrap();

        let url = history.back().unwrap();
        assert_eq!(url, "https://example.com/page1");
        assert_eq!(history.current_index(), 1);
    }

    #[test]
    fn test_back_no_history() {
        let mut history = History::new();
        history.push("https://example.com", None).unwrap();

        let result = history.back();
        assert!(result.is_err());
        match result {
            Err(BrowserError::StorageError(msg)) => {
                assert!(msg.contains("No previous entry"));
            }
            _ => panic!("Expected StorageError"),
        }
    }

    #[test]
    fn test_back_empty_history() {
        let mut history = History::new();
        let result = history.back();
        assert!(result.is_err());
    }

    #[test]
    fn test_forward_navigation() {
        let mut history = History::new();
        history.push("https://example.com/page1", None).unwrap();
        history.push("https://example.com/page2", None).unwrap();
        history.back().unwrap();

        let url = history.forward().unwrap();
        assert_eq!(url, "https://example.com/page2");
        assert_eq!(history.current_index(), 2);
    }

    #[test]
    fn test_forward_no_history() {
        let mut history = History::new();
        history.push("https://example.com/page1", None).unwrap();
        history.push("https://example.com/page2", None).unwrap();

        let result = history.forward();
        assert!(result.is_err());
        match result {
            Err(BrowserError::StorageError(msg)) => {
                assert!(msg.contains("No next entry"));
            }
            _ => panic!("Expected StorageError"),
        }
    }

    #[test]
    fn test_current_url() {
        let mut history = History::new();
        history.push("https://example.com", None).unwrap();

        let url = history.current().unwrap();
        assert_eq!(url, "https://example.com");
    }

    #[test]
    fn test_current_empty_history() {
        let history = History::new();
        let result = history.current();
        assert!(result.is_err());
    }

    #[test]
    fn test_can_go_back() {
        let mut history = History::new();
        assert!(!history.can_go_back());

        history.push("https://example.com/page1", None).unwrap();
        assert!(!history.can_go_back());

        history.push("https://example.com/page2", None).unwrap();
        assert!(history.can_go_back());
    }

    #[test]
    fn test_can_go_forward() {
        let mut history = History::new();
        assert!(!history.can_go_forward());

        history.push("https://example.com/page1", None).unwrap();
        history.push("https://example.com/page2", None).unwrap();
        assert!(!history.can_go_forward());

        history.back().unwrap();
        assert!(history.can_go_forward());
    }

    #[test]
    fn test_clear_history() {
        let mut history = History::new();
        history.push("https://example.com/page1", None).unwrap();
        history.push("https://example.com/page2", None).unwrap();

        history.clear();
        assert_eq!(history.length(), 0);
        assert_eq!(history.current_index(), 0);
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_entries_slice() {
        let mut history = History::new();
        history.push("https://example.com/page1", None).unwrap();
        history.push("https://example.com/page2", None).unwrap();

        let entries = history.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].url(), "https://example.com/page1");
        assert_eq!(entries[1].url(), "https://example.com/page2");
    }

    #[test]
    fn test_history_entry_new() {
        let entry = HistoryEntry::new(
            "https://example.com".to_string(),
            Some("Example".to_string()),
        );
        assert_eq!(entry.url(), "https://example.com");
        assert_eq!(entry.title(), Some("Example"));
    }

    #[test]
    fn test_history_entry_no_title() {
        let entry = HistoryEntry::new("https://example.com".to_string(), None);
        assert_eq!(entry.url(), "https://example.com");
        assert_eq!(entry.title(), None);
    }

    #[test]
    fn test_complex_navigation() {
        let mut history = History::new();
        history.push("https://example.com/page1", None).unwrap();
        history.push("https://example.com/page2", None).unwrap();
        history.push("https://example.com/page3", None).unwrap();

        // Go back twice
        assert_eq!(history.back().unwrap(), "https://example.com/page2");
        assert_eq!(history.back().unwrap(), "https://example.com/page1");

        // Go forward once
        assert_eq!(history.forward().unwrap(), "https://example.com/page2");

        // Add new entry - should discard page3
        history.push("https://example.com/page4", None).unwrap();
        assert_eq!(history.length(), 3);
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_history_display() {
        let mut history = History::new();
        history
            .push("https://example.com/page1", Some("Page 1".to_string()))
            .unwrap();
        history
            .push("https://example.com/page2", Some("Page 2".to_string()))
            .unwrap();

        let display = format!("{}", history);
        assert!(display.contains("History (2 entries)"));
        assert!(display.contains("https://example.com/page1"));
        assert!(display.contains("https://example.com/page2"));
        assert!(display.contains("Page 1"));
        assert!(display.contains("Page 2"));
    }

    #[test]
    fn test_history_max_size() {
        let mut history = History::new();

        // Try to add more than MAX_HISTORY_SIZE entries
        for i in 0..=MAX_HISTORY_SIZE {
            let result = history.push(&format!("https://example.com/page{}", i), None);
            if i < MAX_HISTORY_SIZE {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn test_navigate_back_and_forward_multiple_times() {
        let mut history = History::new();
        history.push("https://example.com/page1", None).unwrap();
        history.push("https://example.com/page2", None).unwrap();
        history.push("https://example.com/page3", None).unwrap();

        // Navigate back and forth
        assert_eq!(history.back().unwrap(), "https://example.com/page2");
        assert_eq!(history.back().unwrap(), "https://example.com/page1");
        assert_eq!(history.forward().unwrap(), "https://example.com/page2");
        assert_eq!(history.forward().unwrap(), "https://example.com/page3");
        assert_eq!(history.back().unwrap(), "https://example.com/page2");
    }

    #[test]
    fn test_current_after_navigation() {
        let mut history = History::new();
        history.push("https://example.com/page1", None).unwrap();
        history.push("https://example.com/page2", None).unwrap();

        history.back().unwrap();
        assert_eq!(history.current().unwrap(), "https://example.com/page1");

        history.forward().unwrap();
        assert_eq!(history.current().unwrap(), "https://example.com/page2");
    }
}
