//! File-based cache storage and retrieval
//!
//! Stores rendered PNG results indexed by SHA-256 hash of content + parameters.
//! Supports TTL-based expiration and LRU-based size eviction.
//! Uses persistent index file to avoid O(n) directory walks during eviction.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Cache entry metadata for the persistent index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheIndexEntry {
    /// Key (SHA-256 hash)
    pub key: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// Last access time (seconds since epoch)
    pub accessed_at: u64,
}

/// Persistent cache index to avoid O(n) directory walks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheIndex {
    /// Entries indexed by key
    entries: BTreeMap<String, CacheIndexEntry>,
}

impl CacheIndex {
    /// Create new empty index
    fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Load index from file
    fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content =
            fs::read_to_string(path)
                .map_err(|e| Error::Cache(format!("Failed to read cache index: {}", e)))?;
        serde_json::from_str(&content)
            .map_err(|e| Error::Cache(format!("Failed to parse cache index: {}", e)))
    }

    /// Save index to file
    fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string(self)
            .map_err(|e| Error::Cache(format!("Failed to serialize index: {}", e)))?;
        fs::write(path, content)
            .map_err(|e| Error::Cache(format!("Failed to write cache index: {}", e)))
    }

    /// Add entry to index
    fn insert(&mut self, entry: CacheIndexEntry) {
        self.entries.insert(entry.key.clone(), entry);
    }

    /// Remove entry from index
    fn remove(&mut self, key: &str) -> Option<CacheIndexEntry> {
        self.entries.remove(key)
    }

    /// Get total size of all entries
    fn total_size(&self) -> u64 {
        self.entries.values().map(|e| e.size_bytes).sum()
    }

    /// Get all entries sorted by access time (oldest first)
    fn entries_by_access_time(&self) -> Vec<(String, u64)> {
        let mut entries: Vec<_> = self
            .entries
            .iter()
            .map(|(k, e)| (k.clone(), e.accessed_at))
            .collect();
        entries.sort_by_key(|(_k, time)| *time);
        entries
    }
}

/// Cache metadata for a stored render
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Creation timestamp (seconds since epoch)
    pub created_at: u64,

    /// TTL in seconds (0 = no expiration)
    pub ttl_secs: u64,

    /// Image width
    pub width: u32,

    /// Image height
    pub height: u32,

    /// Quality setting used
    pub quality: String,

    /// File size in bytes
    pub file_size: u64,
}

impl CacheMetadata {
    /// Check if cache entry is expired
    pub fn is_expired(&self) -> bool {
        if self.ttl_secs == 0 {
            return false; // No expiration
        }

        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(now) => {
                let age_secs = now.as_secs().saturating_sub(self.created_at);
                age_secs > self.ttl_secs
            }
            Err(_) => false, // Clock error, assume not expired
        }
    }
}

/// File-based cache with persistent index
pub struct FileCache {
    cache_dir: PathBuf,
    max_size_mb: u64,
    index_path: PathBuf,
}

impl FileCache {
    /// Create a new file cache
    pub fn new(cache_dir: impl AsRef<Path>, max_size_mb: u64) -> Result<Self> {
        let dir = cache_dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)
            .map_err(|e| Error::Cache(format!("Failed to create cache directory: {}", e)))?;

        let index_path = dir.join(".cache_index.json");

        Ok(Self {
            cache_dir: dir,
            max_size_mb,
            index_path,
        })
    }

    /// Get path to cache index file
    fn get_index_path(&self) -> PathBuf {
        self.index_path.clone()
    }

    /// Compute cache key from content and parameters
    pub fn compute_key(content: &str, quality: &str, image_size: &str) -> String {
        let mut hasher = Sha256::new();

        // Create deterministic input by sorting parameters
        let mut params = BTreeMap::new();
        params.insert("content", content.to_string());
        params.insert("quality", quality.to_string());
        params.insert("image_size", image_size.to_string());

        if let Ok(json) = serde_json::to_string(&params) {
            hasher.update(json);
        }

        format!("{:x}", hasher.finalize())
    }

    /// Get cache entry path
    fn entry_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(key)
    }

    /// Get image file path
    fn image_path(&self, key: &str) -> PathBuf {
        self.entry_path(key).join("image.png")
    }

    /// Get metadata file path
    fn metadata_path(&self, key: &str) -> PathBuf {
        self.entry_path(key).join("metadata.json")
    }

    /// Save image to cache
    pub fn save(&self, key: &str, image_data: &[u8], metadata: &CacheMetadata) -> Result<()> {
        let entry_dir = self.entry_path(key);
        fs::create_dir_all(&entry_dir)
            .map_err(|e| Error::Cache(format!("Failed to create cache entry directory: {}", e)))?;

        // Save image
        let image_path = self.image_path(key);
        fs::write(&image_path, image_data)
            .map_err(|e| Error::Cache(format!("Failed to write cache image: {}", e)))?;

        // Save metadata
        let metadata_path = self.metadata_path(key);
        let metadata_json = serde_json::to_string(metadata)
            .map_err(|e| Error::Cache(format!("Failed to serialize metadata: {}", e)))?;
        fs::write(&metadata_path, metadata_json)
            .map_err(|e| Error::Cache(format!("Failed to write cache metadata: {}", e)))?;

        // Update persistent index
        let mut index = CacheIndex::load_from_file(&self.get_index_path())?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        index.insert(CacheIndexEntry {
            key: key.to_string(),
            size_bytes: image_data.len() as u64,
            accessed_at: now,
        });
        index.save_to_file(&self.get_index_path())?;

        Ok(())
    }

    /// Get image from cache if valid
    pub fn get(&self, key: &str) -> Result<Option<(Vec<u8>, CacheMetadata)>> {
        let image_path = self.image_path(key);
        let metadata_path = self.metadata_path(key);

        // Check if files exist
        if !image_path.exists() || !metadata_path.exists() {
            return Ok(None);
        }

        // Load metadata
        let metadata_json = fs::read_to_string(&metadata_path)
            .map_err(|e| Error::Cache(format!("Failed to read metadata: {}", e)))?;
        let metadata: CacheMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| Error::Cache(format!("Failed to parse metadata: {}", e)))?;

        // Check if expired
        if metadata.is_expired() {
            // Delete expired entry
            let _ = self.delete(key);
            return Ok(None);
        }

        // Load image
        let image_data = fs::read(&image_path)
            .map_err(|e| Error::Cache(format!("Failed to read image: {}", e)))?;

        // Update access time in index
        let mut index = CacheIndex::load_from_file(&self.get_index_path())?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if let Some(mut entry) = index.remove(key) {
            entry.accessed_at = now;
            index.insert(entry);
            let _ = index.save_to_file(&self.get_index_path());
        }

        Ok(Some((image_data, metadata)))
    }

    /// Delete cache entry
    pub fn delete(&self, key: &str) -> Result<()> {
        let entry_dir = self.entry_path(key);
        if entry_dir.exists() {
            fs::remove_dir_all(&entry_dir)
                .map_err(|e| Error::Cache(format!("Failed to delete cache entry: {}", e)))?;
        }

        // Remove from index
        let mut index = CacheIndex::load_from_file(&self.get_index_path())?;
        index.remove(key);
        let _ = index.save_to_file(&self.get_index_path());

        Ok(())
    }

    /// Get cache size in bytes
    pub fn get_size(&self) -> Result<u64> {
        let mut total = 0u64;

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)
                .map_err(|e| Error::Cache(format!("Failed to read cache dir: {}", e)))?
            {
                let entry =
                    entry.map_err(|e| Error::Cache(format!("Failed to read dir entry: {}", e)))?;
                let path = entry.path();

                if path.is_dir() {
                    for file in fs::read_dir(&path)
                        .map_err(|e| Error::Cache(format!("Failed to read subdir: {}", e)))?
                    {
                        let file =
                            file.map_err(|e| Error::Cache(format!("Failed to read file: {}", e)))?;
                        let metadata = file
                            .metadata()
                            .map_err(|e| Error::Cache(format!("Failed to read metadata: {}", e)))?;
                        total += metadata.len();
                    }
                }
            }
        }

        Ok(total)
    }

    /// Evict oldest entries if cache size exceeded
    pub fn evict_if_needed(&self) -> Result<()> {
        let max_bytes = self.max_size_mb * 1024 * 1024;

        // Load index (O(1) file read instead of O(n) directory walk)
        let mut index = CacheIndex::load_from_file(&self.get_index_path())?;
        let current_size = index.total_size();

        if current_size <= max_bytes {
            return Ok(());
        }

        // Get entries sorted by access time (oldest first)
        let entries_by_time = index.entries_by_access_time();

        // Delete oldest entries until under limit
        for (key, _) in entries_by_time {
            if index.total_size() <= max_bytes {
                break;
            }
            // Remove from filesystem
            let entry_path = self.entry_path(&key);
            let _ = fs::remove_dir_all(&entry_path);

            // Remove from index
            index.remove(&key);
        }

        // Save updated index
        index.save_to_file(&self.get_index_path())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_key_same_content() {
        let key1 = FileCache::compute_key("cube(10);", "normal", "800,600");
        let key2 = FileCache::compute_key("cube(10);", "normal", "800,600");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_compute_key_different_content() {
        let key1 = FileCache::compute_key("cube(10);", "normal", "800,600");
        let key2 = FileCache::compute_key("sphere(5);", "normal", "800,600");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_compute_key_different_quality() {
        let key1 = FileCache::compute_key("cube(10);", "draft", "800,600");
        let key2 = FileCache::compute_key("cube(10);", "high", "800,600");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_compute_key_different_size() {
        let key1 = FileCache::compute_key("cube(10);", "normal", "800,600");
        let key2 = FileCache::compute_key("cube(10);", "normal", "1024,768");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_compute_key_is_hex() {
        let key = FileCache::compute_key("test", "normal", "800,600");
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_cache_metadata_not_expired() {
        let metadata = CacheMetadata {
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl_secs: 3600,
            width: 800,
            height: 600,
            quality: "normal".to_string(),
            file_size: 1024,
        };
        assert!(!metadata.is_expired());
    }

    #[test]
    fn test_cache_metadata_no_expiration() {
        let metadata = CacheMetadata {
            created_at: 0,
            ttl_secs: 0,
            width: 800,
            height: 600,
            quality: "normal".to_string(),
            file_size: 1024,
        };
        assert!(!metadata.is_expired());
    }

    #[test]
    fn test_file_cache_new() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = FileCache::new(temp_dir.path(), 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_cache_save_and_get() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = FileCache::new(temp_dir.path(), 100).unwrap();

        let key = "test_key";
        let image_data = b"fake png data";
        let metadata = CacheMetadata {
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl_secs: 3600,
            width: 800,
            height: 600,
            quality: "normal".to_string(),
            file_size: image_data.len() as u64,
        };

        let save_result = cache.save(key, image_data, &metadata);
        assert!(save_result.is_ok());

        let get_result = cache.get(key).unwrap();
        assert!(get_result.is_some());
        let (data, _) = get_result.unwrap();
        assert_eq!(data, image_data);
    }

    #[test]
    fn test_file_cache_get_nonexistent() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = FileCache::new(temp_dir.path(), 100).unwrap();

        let get_result = cache.get("nonexistent").unwrap();
        assert!(get_result.is_none());
    }

    #[test]
    fn test_file_cache_delete() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = FileCache::new(temp_dir.path(), 100).unwrap();

        let key = "test_key";
        let image_data = b"test data";
        let metadata = CacheMetadata {
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl_secs: 3600,
            width: 800,
            height: 600,
            quality: "normal".to_string(),
            file_size: image_data.len() as u64,
        };

        cache.save(key, image_data, &metadata).unwrap();
        assert!(cache.get(key).unwrap().is_some());

        cache.delete(key).unwrap();
        assert!(cache.get(key).unwrap().is_none());
    }

    #[test]
    fn test_file_cache_get_size() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = FileCache::new(temp_dir.path(), 100).unwrap();

        let size1 = cache.get_size().unwrap();
        assert_eq!(size1, 0);

        let key = "test_key";
        let image_data = b"test data here";
        let metadata = CacheMetadata {
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl_secs: 3600,
            width: 800,
            height: 600,
            quality: "normal".to_string(),
            file_size: image_data.len() as u64,
        };

        cache.save(key, image_data, &metadata).unwrap();
        let size2 = cache.get_size().unwrap();
        assert!(size2 > size1);
    }

    #[test]
    fn test_file_cache_evict_if_needed() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache = FileCache::new(temp_dir.path(), 1).unwrap(); // 1 MB limit

        let key = "test_key";
        let image_data = vec![0u8; 512 * 1024]; // 512 KB
        let metadata = CacheMetadata {
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl_secs: 3600,
            width: 800,
            height: 600,
            quality: "normal".to_string(),
            file_size: image_data.len() as u64,
        };

        cache.save(key, &image_data, &metadata).unwrap();
        let result = cache.evict_if_needed();
        assert!(result.is_ok());
    }

    #[test]
    fn test_cache_metadata_serialization() {
        let metadata = CacheMetadata {
            created_at: 1234567890,
            ttl_secs: 3600,
            width: 1024,
            height: 768,
            quality: "high".to_string(),
            file_size: 2048,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: CacheMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata.created_at, deserialized.created_at);
    }
}
