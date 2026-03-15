//! Configuration management module
//!
//! Handles loading and managing configuration from environment variables,
//! YAML files, and .env files.

pub mod loader;

pub use loader::{Config, ConfigBuilder};
