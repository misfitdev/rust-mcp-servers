//! Caching module
//!
//! Provides file-based caching for rendered outputs with TTL and LRU eviction.

pub mod file_cache;
pub mod metrics;

pub use file_cache::{CacheMetadata, FileCache};
pub use metrics::CacheMetrics;
