//! Custom error types for OpenSCAD MCP Server
//!
//! Provides a unified error handling interface covering rendering,
//! caching, filesystem, and validation failures.

use std::io;
use thiserror::Error;

/// Result type alias for operations that may fail with an OpenSCAD error
pub type Result<T> = std::result::Result<T, Error>;

/// Unified error type for all OpenSCAD MCP operations
#[derive(Error, Debug)]
pub enum Error {
    /// OpenSCAD rendering failed
    #[error("render error: {0}")]
    Render(String),

    /// Cache operation failed
    #[error("cache error: {0}")]
    Cache(String),

    /// Filesystem operation failed
    #[error("filesystem error: {0}")]
    Filesystem(#[from] io::Error),

    /// Validation failed
    #[error("validation error: {0}")]
    Validation(String),

    /// Configuration error
    #[error("config error: {0}")]
    Config(String),

    /// OpenSCAD executable not found
    #[error("openscad executable not found: {0}")]
    OpenSCADNotFound(String),

    /// Serialization/Deserialization error
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Generic error
    #[error("error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_error() {
        let err = Error::Render("Failed to render model".to_string());
        assert_eq!(err.to_string(), "render error: Failed to render model");
    }

    #[test]
    fn test_cache_error() {
        let err = Error::Cache("Cache miss".to_string());
        assert_eq!(err.to_string(), "cache error: Cache miss");
    }

    #[test]
    fn test_validation_error() {
        let err = Error::Validation("Invalid camera position".to_string());
        assert_eq!(err.to_string(), "validation error: Invalid camera position");
    }

    #[test]
    fn test_openscad_not_found() {
        let err = Error::OpenSCADNotFound("/usr/bin/openscad".to_string());
        assert!(err.to_string().contains("openscad executable not found"));
    }

    #[test]
    fn test_config_error() {
        let err = Error::Config("Missing required config key".to_string());
        assert_eq!(err.to_string(), "config error: Missing required config key");
    }

    #[test]
    fn test_serialization_error() {
        let err = Error::Serialization("Failed to parse JSON".to_string());
        assert_eq!(err.to_string(), "serialization error: Failed to parse JSON");
    }

    #[test]
    fn test_filesystem_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }
        assert_eq!(returns_result().unwrap(), 42);
    }

    #[test]
    fn test_result_error_propagation() {
        fn returns_error() -> Result<i32> {
            Err(Error::Validation("bad input".to_string()))
        }
        match returns_error() {
            Err(Error::Validation(msg)) => assert_eq!(msg, "bad input"),
            _ => panic!("Expected validation error"),
        }
    }
}
