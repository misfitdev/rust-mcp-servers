//! Cache metrics and statistics
//!
//! Tracks cache performance metrics: hits, misses, size, entry count.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Cache performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Number of cache hits
    pub hits: u64,

    /// Number of cache misses
    pub misses: u64,

    /// Current cache size in bytes
    pub size_bytes: u64,

    /// Number of entries in cache
    pub entry_count: u64,
}

impl CacheMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            size_bytes: 0,
            entry_count: 0,
        }
    }

    /// Get hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Get cache size in MB
    pub fn size_mb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Record a cache hit
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Record a cache miss
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// Update cache size
    pub fn set_size(&mut self, size_bytes: u64) {
        self.size_bytes = size_bytes;
    }

    /// Update entry count
    pub fn set_entry_count(&mut self, count: u64) {
        self.entry_count = count;
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.size_bytes = 0;
        self.entry_count = 0;
    }
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe cache metrics
pub struct AtomicCacheMetrics {
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    size_bytes: Arc<AtomicU64>,
    entry_count: Arc<AtomicU64>,
}

impl AtomicCacheMetrics {
    /// Create new atomic metrics
    pub fn new() -> Self {
        Self {
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            size_bytes: Arc::new(AtomicU64::new(0)),
            entry_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a hit
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a miss
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Update size
    pub fn set_size(&self, size: u64) {
        self.size_bytes.store(size, Ordering::Relaxed);
    }

    /// Update entry count
    pub fn set_entry_count(&self, count: u64) {
        self.entry_count.store(count, Ordering::Relaxed);
    }

    /// Get snapshot of metrics
    pub fn snapshot(&self) -> CacheMetrics {
        CacheMetrics {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            size_bytes: self.size_bytes.load(Ordering::Relaxed),
            entry_count: self.entry_count.load(Ordering::Relaxed),
        }
    }
}

impl Default for AtomicCacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_new() {
        let metrics = CacheMetrics::new();
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.misses, 0);
        assert_eq!(metrics.size_bytes, 0);
        assert_eq!(metrics.entry_count, 0);
    }

    #[test]
    fn test_metrics_hit_rate_no_data() {
        let metrics = CacheMetrics::new();
        assert_eq!(metrics.hit_rate(), 0.0);
    }

    #[test]
    fn test_metrics_hit_rate_perfect() {
        let metrics = CacheMetrics {
            hits: 10,
            misses: 0,
            size_bytes: 1024,
            entry_count: 5,
        };
        assert_eq!(metrics.hit_rate(), 100.0);
    }

    #[test]
    fn test_metrics_hit_rate_half() {
        let metrics = CacheMetrics {
            hits: 5,
            misses: 5,
            size_bytes: 1024,
            entry_count: 5,
        };
        assert_eq!(metrics.hit_rate(), 50.0);
    }

    #[test]
    fn test_metrics_size_mb() {
        let metrics = CacheMetrics {
            hits: 0,
            misses: 0,
            size_bytes: 1024 * 1024,
            entry_count: 0,
        };
        assert_eq!(metrics.size_mb(), 1.0);
    }

    #[test]
    fn test_metrics_record_hit() {
        let mut metrics = CacheMetrics::new();
        metrics.record_hit();
        assert_eq!(metrics.hits, 1);
    }

    #[test]
    fn test_metrics_record_miss() {
        let mut metrics = CacheMetrics::new();
        metrics.record_miss();
        assert_eq!(metrics.misses, 1);
    }

    #[test]
    fn test_metrics_reset() {
        let mut metrics = CacheMetrics {
            hits: 10,
            misses: 5,
            size_bytes: 2048,
            entry_count: 3,
        };
        metrics.reset();
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.misses, 0);
        assert_eq!(metrics.size_bytes, 0);
        assert_eq!(metrics.entry_count, 0);
    }

    #[test]
    fn test_metrics_serialization() {
        let metrics = CacheMetrics {
            hits: 10,
            misses: 5,
            size_bytes: 1024,
            entry_count: 3,
        };
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: CacheMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics.hits, deserialized.hits);
    }

    #[test]
    fn test_atomic_metrics_new() {
        let metrics = AtomicCacheMetrics::new();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.hits, 0);
    }

    #[test]
    fn test_atomic_metrics_record_hit() {
        let metrics = AtomicCacheMetrics::new();
        metrics.record_hit();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.hits, 1);
    }

    #[test]
    fn test_atomic_metrics_record_miss() {
        let metrics = AtomicCacheMetrics::new();
        metrics.record_miss();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.misses, 1);
    }

    #[test]
    fn test_atomic_metrics_set_size() {
        let metrics = AtomicCacheMetrics::new();
        metrics.set_size(2048);
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.size_bytes, 2048);
    }

    #[test]
    fn test_atomic_metrics_set_entry_count() {
        let metrics = AtomicCacheMetrics::new();
        metrics.set_entry_count(5);
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.entry_count, 5);
    }

    #[test]
    fn test_cache_metrics_default() {
        let metrics = CacheMetrics::default();
        assert_eq!(metrics.hits, 0);
    }
}
