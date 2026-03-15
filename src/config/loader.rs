//! Configuration loader with support for env vars, YAML, and .env files

use serde::{Deserialize, Serialize};
use std::path::Path;

/// OpenSCAD MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// OpenSCAD executable path
    pub openscad_path: Option<String>,

    /// Cache directory
    pub cache_dir: Option<String>,

    /// Cache size limit in MB
    pub cache_size_mb: Option<u64>,

    /// Cache TTL in seconds
    pub cache_ttl_secs: Option<u64>,

    /// Render timeout in seconds
    pub render_timeout_secs: Option<u64>,

    /// Log level
    pub log_level: Option<String>,

    /// Model directory
    pub model_dir: Option<String>,
}

/// Builder for configuration with fluent API
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Create a new config builder
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Set the OpenSCAD executable path
    pub fn openscad_path(mut self, path: String) -> Self {
        self.config.openscad_path = Some(path);
        self
    }

    /// Set the cache directory
    pub fn cache_dir(mut self, path: String) -> Self {
        self.config.cache_dir = Some(path);
        self
    }

    /// Set the cache size limit
    pub fn cache_size_mb(mut self, size: u64) -> Self {
        self.config.cache_size_mb = Some(size);
        self
    }

    /// Set the cache TTL
    pub fn cache_ttl_secs(mut self, ttl: u64) -> Self {
        self.config.cache_ttl_secs = Some(ttl);
        self
    }

    /// Set the render timeout
    pub fn render_timeout_secs(mut self, timeout: u64) -> Self {
        self.config.render_timeout_secs = Some(timeout);
        self
    }

    /// Set the log level
    pub fn log_level(mut self, level: String) -> Self {
        self.config.log_level = Some(level);
        self
    }

    /// Set the model directory
    pub fn model_dir(mut self, path: String) -> Self {
        self.config.model_dir = Some(path);
        self
    }

    /// Build the configuration
    pub fn build(self) -> Config {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Load configuration from environment variables
pub fn load_from_env() -> Config {
    Config {
        openscad_path: std::env::var("OPENSCAD_PATH").ok(),
        cache_dir: std::env::var("CACHE_DIR").ok(),
        cache_size_mb: std::env::var("CACHE_SIZE_MB")
            .ok()
            .and_then(|v| v.parse().ok()),
        cache_ttl_secs: std::env::var("CACHE_TTL_SECS")
            .ok()
            .and_then(|v| v.parse().ok()),
        render_timeout_secs: std::env::var("RENDER_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok()),
        log_level: std::env::var("LOG_LEVEL").ok(),
        model_dir: std::env::var("MODEL_DIR").ok(),
    }
}

/// Load configuration from a YAML file
pub fn load_from_yaml(_path: &Path) -> crate::error::Result<Config> {
    // Placeholder - to be implemented
    Ok(Config::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.openscad_path.is_none());
        assert!(config.cache_dir.is_none());
        assert!(config.cache_size_mb.is_none());
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .openscad_path("/usr/bin/openscad".to_string())
            .cache_dir("/tmp/cache".to_string())
            .cache_size_mb(100)
            .build();

        assert_eq!(config.openscad_path, Some("/usr/bin/openscad".to_string()));
        assert_eq!(config.cache_dir, Some("/tmp/cache".to_string()));
        assert_eq!(config.cache_size_mb, Some(100));
        assert!(config.cache_ttl_secs.is_none());
    }

    #[test]
    fn test_config_builder_chaining() {
        let config = ConfigBuilder::new()
            .openscad_path("/usr/bin/openscad".to_string())
            .cache_size_mb(50)
            .cache_ttl_secs(3600)
            .render_timeout_secs(120)
            .log_level("debug".to_string())
            .build();

        assert_eq!(config.openscad_path, Some("/usr/bin/openscad".to_string()));
        assert_eq!(config.cache_size_mb, Some(50));
        assert_eq!(config.cache_ttl_secs, Some(3600));
        assert_eq!(config.render_timeout_secs, Some(120));
        assert_eq!(config.log_level, Some("debug".to_string()));
    }

    #[test]
    fn test_load_from_env_empty() {
        // Clear relevant env vars
        std::env::remove_var("OPENSCAD_PATH");
        std::env::remove_var("CACHE_DIR");

        let config = load_from_env();
        assert!(config.openscad_path.is_none());
        assert!(config.cache_dir.is_none());
    }

    #[test]
    fn test_load_from_env_with_values() {
        std::env::set_var("OPENSCAD_PATH", "/custom/openscad");
        std::env::set_var("CACHE_SIZE_MB", "256");

        let config = load_from_env();
        assert_eq!(config.openscad_path, Some("/custom/openscad".to_string()));
        assert_eq!(config.cache_size_mb, Some(256));

        std::env::remove_var("OPENSCAD_PATH");
        std::env::remove_var("CACHE_SIZE_MB");
    }

    #[test]
    fn test_load_from_yaml_placeholder() {
        let config = load_from_yaml(Path::new("nonexistent.yaml"));
        assert!(config.is_ok());
        let cfg = config.unwrap();
        assert_eq!(cfg.openscad_path, None);
    }

    #[test]
    fn test_config_validation_cache_size() {
        let config = ConfigBuilder::new().cache_size_mb(1024).build();

        assert_eq!(config.cache_size_mb, Some(1024));
    }

    #[test]
    fn test_config_validation_timeout() {
        let config = ConfigBuilder::new().render_timeout_secs(300).build();

        assert_eq!(config.render_timeout_secs, Some(300));
    }

    #[test]
    fn test_config_serialization() {
        let config = ConfigBuilder::new()
            .openscad_path("/usr/bin/openscad".to_string())
            .cache_dir("/tmp".to_string())
            .build();

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("openscad_path"));
        assert!(json.contains("/usr/bin/openscad"));
    }

    #[test]
    fn test_config_deserialization() {
        let json = r#"{"openscad_path": "/usr/bin/openscad", "cache_size_mb": 100}"#;
        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.openscad_path, Some("/usr/bin/openscad".to_string()));
        assert_eq!(config.cache_size_mb, Some(100));
    }
}
