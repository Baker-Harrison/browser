//! Storage engine for browser persistence
//!
//! This module provides the LocalStorage implementation for persistent key-value storage.
//! Data is stored in memory and persisted to disk using JSON format.

use crate::error::{BrowserError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// In-memory and persistent storage for key-value pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageData {
    /// The key-value pairs
    pub data: HashMap<String, String>,
}

impl Default for StorageData {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageData {
    /// Create a new empty storage
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Load storage data from a file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            // Return empty storage if file doesn't exist
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| BrowserError::StorageReadError(format!("Failed to read file: {}", e)))?;

        if content.trim().is_empty() {
            return Ok(Self::new());
        }

        serde_json::from_str(&content)
            .map_err(|e| BrowserError::StorageReadError(format!("Failed to parse JSON: {}", e)))
    }

    /// Save storage data to a file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                BrowserError::StorageWriteError(format!("Failed to create directory: {}", e))
            })?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| BrowserError::StorageWriteError(format!("Failed to serialize: {}", e)))?;

        fs::write(path, json)
            .map_err(|e| BrowserError::StorageWriteError(format!("Failed to write file: {}", e)))
    }
}

/// LocalStorage implementation for browser persistence
#[derive(Debug)]
pub struct LocalStorageImpl {
    /// The storage data
    data: StorageData,
    /// Path to the storage file on disk
    storage_path: PathBuf,
    /// Whether to auto-persist on every modification
    auto_persist: bool,
}

impl LocalStorageImpl {
    /// Create a new LocalStorage with the given storage path
    ///
    /// # Arguments
    /// * `storage_path` - Path to the file where data will be persisted
    /// * `auto_persist` - If true, automatically save to disk on every modification
    ///
    /// # Errors
    /// Returns an error if the storage file cannot be loaded
    pub fn new<P: AsRef<Path>>(storage_path: P, auto_persist: bool) -> Result<Self> {
        let path = storage_path.as_ref();
        let data = StorageData::load_from_file(path)?;

        Ok(Self {
            data,
            storage_path: path.to_path_buf(),
            auto_persist,
        })
    }

    /// Create a new in-memory LocalStorage (no persistence)
    pub fn new_memory() -> Self {
        Self {
            data: StorageData::new(),
            storage_path: PathBuf::from(":memory:"),
            auto_persist: false,
        }
    }

    /// Persist the current state to disk
    pub fn persist(&self) -> Result<()> {
        if self.storage_path.as_os_str() == ":memory:" {
            return Ok(()); // No persistence for in-memory storage
        }

        self.data.save_to_file(&self.storage_path)
    }

    /// Internal method to persist if auto_persist is enabled
    fn auto_persist_if_enabled(&self) -> Result<()> {
        if self.auto_persist {
            self.persist()
        } else {
            Ok(())
        }
    }
}

/// Trait for LocalStorage operations
pub trait LocalStorage {
    /// Get the value for a given key. Returns None if key doesn't exist.
    fn get_item(&self, key: &str) -> Result<Option<String>>;

    /// Set a key-value pair. Overwrites existing value if key exists.
    fn set_item(&mut self, key: &str, value: &str) -> Result<()>;

    /// Remove a key-value pair. Does nothing if key doesn't exist.
    fn remove_item(&mut self, key: &str) -> Result<()>;

    /// Clear all key-value pairs.
    fn clear(&mut self) -> Result<()>;

    /// Get the number of key-value pairs.
    fn len(&self) -> usize;

    /// Check if storage is empty.
    fn is_empty(&self) -> bool;

    /// Get all keys in storage.
    fn keys(&self) -> Vec<String>;

    /// Persist current state to disk.
    fn persist(&self) -> Result<()>;
}

impl LocalStorage for LocalStorageImpl {
    fn get_item(&self, key: &str) -> Result<Option<String>> {
        Ok(self.data.data.get(key).cloned())
    }

    fn set_item(&mut self, key: &str, value: &str) -> Result<()> {
        self.data.data.insert(key.to_string(), value.to_string());
        self.auto_persist_if_enabled()
    }

    fn remove_item(&mut self, key: &str) -> Result<()> {
        self.data.data.remove(key);
        self.auto_persist_if_enabled()
    }

    fn clear(&mut self) -> Result<()> {
        self.data.data.clear();
        self.auto_persist_if_enabled()
    }

    fn len(&self) -> usize {
        self.data.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.data.is_empty()
    }

    fn keys(&self) -> Vec<String> {
        self.data.data.keys().cloned().collect()
    }

    fn persist(&self) -> Result<()> {
        self.persist()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_storage_data_new() {
        let data = StorageData::new();
        assert!(data.data.is_empty());
    }

    #[test]
    fn test_storage_data_default() {
        let data = StorageData::default();
        assert!(data.data.is_empty());
    }

    #[test]
    fn test_storage_data_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("storage.json");

        let mut data = StorageData::new();
        data.data.insert("key1".to_string(), "value1".to_string());
        data.data.insert("key2".to_string(), "value2".to_string());

        data.save_to_file(&path).unwrap();
        assert!(path.exists());

        let loaded = StorageData::load_from_file(&path).unwrap();
        assert_eq!(loaded.data.len(), 2);
        assert_eq!(loaded.data.get("key1"), Some(&"value1".to_string()));
        assert_eq!(loaded.data.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_storage_data_load_nonexistent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");

        let data = StorageData::load_from_file(&path).unwrap();
        assert!(data.data.is_empty());
    }

    #[test]
    fn test_storage_data_load_empty_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.json");
        fs::write(&path, "").unwrap();

        let data = StorageData::load_from_file(&path).unwrap();
        assert!(data.data.is_empty());
    }

    #[test]
    fn test_local_storage_memory() {
        let mut storage = LocalStorageImpl::new_memory();

        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);

        storage.set_item("key1", "value1").unwrap();
        assert_eq!(storage.len(), 1);
        assert!(!storage.is_empty());

        let value = storage.get_item("key1").unwrap();
        assert_eq!(value, Some("value1".to_string()));

        let missing = storage.get_item("nonexistent").unwrap();
        assert_eq!(missing, None);
    }

    #[test]
    fn test_local_storage_persistent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("storage.json");

        let mut storage = LocalStorageImpl::new(&path, false).unwrap();
        assert!(storage.is_empty());

        storage.set_item("key1", "value1").unwrap();
        storage.set_item("key2", "value2").unwrap();
        storage.persist().unwrap();

        // Load a new instance from the same path
        let storage2 = LocalStorageImpl::new(&path, false).unwrap();
        assert_eq!(storage2.len(), 2);
        assert_eq!(
            storage2.get_item("key1").unwrap(),
            Some("value1".to_string())
        );
        assert_eq!(
            storage2.get_item("key2").unwrap(),
            Some("value2".to_string())
        );
    }

    #[test]
    fn test_local_storage_auto_persist() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("storage.json");

        let mut storage = LocalStorageImpl::new(&path, true).unwrap();
        storage.set_item("key1", "value1").unwrap();

        // Load a new instance - should have the data due to auto-persist
        let storage2 = LocalStorageImpl::new(&path, false).unwrap();
        assert_eq!(
            storage2.get_item("key1").unwrap(),
            Some("value1".to_string())
        );
    }

    #[test]
    fn test_local_storage_remove_item() {
        let mut storage = LocalStorageImpl::new_memory();

        storage.set_item("key1", "value1").unwrap();
        storage.set_item("key2", "value2").unwrap();
        assert_eq!(storage.len(), 2);

        storage.remove_item("key1").unwrap();
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.get_item("key1").unwrap(), None);
        assert_eq!(
            storage.get_item("key2").unwrap(),
            Some("value2".to_string())
        );

        // Removing non-existent key should not error
        storage.remove_item("nonexistent").unwrap();
        assert_eq!(storage.len(), 1);
    }

    #[test]
    fn test_local_storage_clear() {
        let mut storage = LocalStorageImpl::new_memory();

        storage.set_item("key1", "value1").unwrap();
        storage.set_item("key2", "value2").unwrap();
        storage.set_item("key3", "value3").unwrap();
        assert_eq!(storage.len(), 3);

        storage.clear().unwrap();
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_local_storage_keys() {
        let mut storage = LocalStorageImpl::new_memory();

        storage.set_item("key1", "value1").unwrap();
        storage.set_item("key2", "value2").unwrap();
        storage.set_item("key3", "value3").unwrap();

        let keys = storage.keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }

    #[test]
    fn test_local_storage_overwrite() {
        let mut storage = LocalStorageImpl::new_memory();

        storage.set_item("key1", "value1").unwrap();
        assert_eq!(
            storage.get_item("key1").unwrap(),
            Some("value1".to_string())
        );

        storage.set_item("key1", "value2").unwrap();
        assert_eq!(
            storage.get_item("key1").unwrap(),
            Some("value2".to_string())
        );
        assert_eq!(storage.len(), 1);
    }

    #[test]
    fn test_local_storage_empty_values() {
        let mut storage = LocalStorageImpl::new_memory();

        storage.set_item("key1", "").unwrap();
        assert_eq!(storage.get_item("key1").unwrap(), Some("".to_string()));
        assert_eq!(storage.len(), 1);
    }

    #[test]
    fn test_local_storage_special_characters() {
        let mut storage = LocalStorageImpl::new_memory();

        storage
            .set_item("key with spaces", "value with spaces")
            .unwrap();
        storage
            .set_item("key\nwith\nnewlines", "value\nwith\nnewlines")
            .unwrap();
        storage
            .set_item("key\"with\"quotes", "value\"with\"quotes")
            .unwrap();

        assert_eq!(
            storage.get_item("key with spaces").unwrap(),
            Some("value with spaces".to_string())
        );
        assert_eq!(
            storage.get_item("key\nwith\nnewlines").unwrap(),
            Some("value\nwith\nnewlines".to_string())
        );
        assert_eq!(
            storage.get_item("key\"with\"quotes").unwrap(),
            Some("value\"with\"quotes".to_string())
        );
    }

    #[test]
    fn test_local_storage_unicode() {
        let mut storage = LocalStorageImpl::new_memory();

        storage.set_item("key🔑", "value🎉").unwrap();
        storage.set_item("键", "值").unwrap();
        storage.set_item("ключ", "значение").unwrap();

        assert_eq!(
            storage.get_item("key🔑").unwrap(),
            Some("value🎉".to_string())
        );
        assert_eq!(storage.get_item("键").unwrap(), Some("值".to_string()));
        assert_eq!(
            storage.get_item("ключ").unwrap(),
            Some("значение".to_string())
        );
    }

    #[test]
    fn test_local_storage_persist_creates_directory() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("subdir").join("storage.json");

        let mut storage = LocalStorageImpl::new(&path, false).unwrap();
        storage.set_item("key1", "value1").unwrap();
        storage.persist().unwrap();

        assert!(path.exists());
        assert!(path.parent().unwrap().exists());
    }

    #[test]
    fn test_local_storage_trait_object() {
        let mut storage: Box<dyn LocalStorage> = Box::new(LocalStorageImpl::new_memory());

        storage.set_item("key1", "value1").unwrap();
        assert_eq!(
            storage.get_item("key1").unwrap(),
            Some("value1".to_string())
        );
        assert_eq!(storage.len(), 1);

        storage.remove_item("key1").unwrap();
        assert_eq!(storage.len(), 0);
    }
}
