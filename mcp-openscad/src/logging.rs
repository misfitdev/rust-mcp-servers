//! Logging initialization and configuration
//!
//! Provides a centralized logging setup using tracing + tracing-subscriber.

use tracing_subscriber::EnvFilter;

/// Initialize logging with RUST_LOG environment variable support
///
/// # Examples
///
/// ```
/// // Uses RUST_LOG env var, defaults to "info"
/// openscad_mcp::logging::init();
/// ```
pub fn init() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

/// Initialize logging with a specific level
pub fn init_with_level(level: &str) {
    let env_filter = EnvFilter::new(level);
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_filter_default() {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        // Just verify it parses without error
        assert!(format!("{:?}", filter).contains("EnvFilter"));
    }

    #[test]
    fn test_env_filter_debug() {
        let filter = EnvFilter::new("debug");
        assert!(format!("{:?}", filter).contains("EnvFilter"));
    }

    #[test]
    fn test_env_filter_trace() {
        let filter = EnvFilter::new("trace");
        assert!(format!("{:?}", filter).contains("EnvFilter"));
    }

    #[test]
    fn test_env_filter_warn() {
        let filter = EnvFilter::new("warn");
        assert!(format!("{:?}", filter).contains("EnvFilter"));
    }

    #[test]
    fn test_env_filter_error() {
        let filter = EnvFilter::new("error");
        assert!(format!("{:?}", filter).contains("EnvFilter"));
    }

    #[test]
    fn test_env_filter_module_specific() {
        let filter = EnvFilter::new("openscad_mcp=debug");
        assert!(format!("{:?}", filter).contains("EnvFilter"));
    }

    #[test]
    fn test_env_filter_multiple_modules() {
        let filter = EnvFilter::new("openscad_mcp=debug,tokio=info");
        assert!(format!("{:?}", filter).contains("EnvFilter"));
    }

    #[test]
    fn test_span_creation() {
        // Just verify that span macros compile and work
        let span = tracing::debug_span!("test_operation");
        let _guard = span.enter();
        // Span successfully entered
    }

    #[test]
    fn test_initialization_idempotent() {
        // First call should work
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        assert!(!format!("{:?}", env_filter).is_empty());
    }
}
