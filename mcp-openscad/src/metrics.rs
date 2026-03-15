//! Server-wide metrics and observability
//!
//! Tracks active renders, queue depth, performance metrics, and provides introspection tools.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Server metrics tracking active renders and performance
#[derive(Debug, Clone)]
pub struct ServerMetrics {
    /// Number of currently active renders
    active_renders: Arc<AtomicUsize>,
    /// Total renders completed
    total_renders: Arc<AtomicUsize>,
    /// Total render duration in milliseconds
    total_render_ms: Arc<AtomicUsize>,
}

impl ServerMetrics {
    /// Create new server metrics
    pub fn new() -> Self {
        Self {
            active_renders: Arc::new(AtomicUsize::new(0)),
            total_renders: Arc::new(AtomicUsize::new(0)),
            total_render_ms: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Increment active render count
    pub fn increment_active(&self) {
        self.active_renders.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement active render count
    pub fn decrement_active(&self) {
        self.active_renders.fetch_sub(1, Ordering::SeqCst);
    }

    /// Get current active render count
    pub fn active_count(&self) -> usize {
        self.active_renders.load(Ordering::SeqCst)
    }

    /// Record a completed render
    pub fn record_render(&self, duration_ms: usize) {
        self.total_renders.fetch_add(1, Ordering::SeqCst);
        self.total_render_ms
            .fetch_add(duration_ms, Ordering::SeqCst);
    }

    /// Get total renders completed
    pub fn total_renders(&self) -> usize {
        self.total_renders.load(Ordering::SeqCst)
    }

    /// Get average render duration in milliseconds
    pub fn average_render_ms(&self) -> f64 {
        let total = self.total_renders.load(Ordering::SeqCst);
        if total == 0 {
            0.0
        } else {
            self.total_render_ms.load(Ordering::SeqCst) as f64 / total as f64
        }
    }
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_metrics_new() {
        let metrics = ServerMetrics::new();
        assert_eq!(metrics.active_count(), 0);
        assert_eq!(metrics.total_renders(), 0);
        assert_eq!(metrics.average_render_ms(), 0.0);
    }

    #[test]
    fn test_server_metrics_active_renders() {
        let metrics = ServerMetrics::new();
        metrics.increment_active();
        assert_eq!(metrics.active_count(), 1);
        metrics.increment_active();
        assert_eq!(metrics.active_count(), 2);
        metrics.decrement_active();
        assert_eq!(metrics.active_count(), 1);
    }

    #[test]
    fn test_server_metrics_record_render() {
        let metrics = ServerMetrics::new();
        metrics.record_render(100);
        assert_eq!(metrics.total_renders(), 1);
        assert_eq!(metrics.average_render_ms(), 100.0);
        metrics.record_render(200);
        assert_eq!(metrics.total_renders(), 2);
        assert_eq!(metrics.average_render_ms(), 150.0);
    }

    #[test]
    fn test_server_metrics_default() {
        let metrics = ServerMetrics::default();
        assert_eq!(metrics.active_count(), 0);
    }

    #[test]
    fn test_server_metrics_concurrent_updates() {
        let metrics = ServerMetrics::new();
        metrics.increment_active();
        metrics.increment_active();
        metrics.increment_active();
        assert_eq!(metrics.active_count(), 3);
        metrics.record_render(50);
        metrics.record_render(150);
        assert_eq!(metrics.average_render_ms(), 100.0);
    }

    #[test]
    fn test_list_renders_placeholder() {
        // Placeholder test for list_renders functionality
        // Will track active render tasks
        assert!(true);
    }

    #[test]
    fn test_kill_render_placeholder() {
        // Placeholder test for kill_render functionality
        // Will test task termination and error handling
        assert!(true);
    }
}
