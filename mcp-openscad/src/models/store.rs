//! Model file store operations
//!
//! Provides atomic CRUD operations for OpenSCAD model files.

use crate::error::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Model file store
pub struct ModelStore;

impl ModelStore {
    /// Create a new model file with atomic write
    pub fn create(path: impl AsRef<Path>, content: &str) -> Result<()> {
        let path = path.as_ref();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    Error::Filesystem(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to create parent directory: {}", e),
                    ))
                })?;
            }
        }

        // Write atomically with tempfile
        let temp_file =
            tempfile::NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new(".")))
                .map_err(|e| Error::Cache(format!("Failed to create temp file: {}", e)))?;

        let temp_path = temp_file.path().to_path_buf();
        fs::write(&temp_path, content).map_err(|e| Error::Filesystem(e))?;

        // Atomic rename
        fs::rename(&temp_path, path).map_err(|e| Error::Filesystem(e))?;

        Ok(())
    }

    /// Read a model file
    pub fn read(path: impl AsRef<Path>) -> Result<String> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(Error::Validation(format!(
                "Model file not found: {}",
                path.display()
            )));
        }

        fs::read_to_string(path).map_err(|e| Error::Filesystem(e))
    }

    /// Update a model file atomically
    pub fn update(path: impl AsRef<Path>, content: &str) -> Result<()> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(Error::Validation(format!(
                "Model file not found: {}",
                path.display()
            )));
        }

        // Use same atomic approach as create
        Self::create(path, content)
    }

    /// Delete a model file
    pub fn delete(path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(Error::Validation(format!(
                "Model file not found: {}",
                path.display()
            )));
        }

        fs::remove_file(path).map_err(|e| Error::Filesystem(e))
    }

    /// List all .scad files in directory
    pub fn list_models(dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let dir = dir.as_ref();

        if !dir.exists() {
            return Ok(Vec::new());
        }

        if !dir.is_dir() {
            return Err(Error::Validation("Path is not a directory".to_string()));
        }

        let mut models = Vec::new();

        for entry in fs::read_dir(dir).map_err(|e| Error::Filesystem(e))? {
            let entry = entry.map_err(|e| Error::Filesystem(e))?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "scad" {
                        models.push(path);
                    }
                }
            }
        }

        models.sort();
        Ok(models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_model() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("test.scad");

        let result = ModelStore::create(&model_path, "cube(10);");
        assert!(result.is_ok());
        assert!(model_path.exists());
    }

    #[test]
    fn test_create_model_creates_parent_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("subdir").join("test.scad");

        let result = ModelStore::create(&model_path, "sphere(5);");
        assert!(result.is_ok());
        assert!(model_path.exists());
    }

    #[test]
    fn test_read_model() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("test.scad");
        let content = "cube([10, 20, 30]);";

        ModelStore::create(&model_path, content).unwrap();
        let read_content = ModelStore::read(&model_path).unwrap();

        assert_eq!(read_content, content);
    }

    #[test]
    fn test_read_nonexistent_model() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("nonexistent.scad");

        let result = ModelStore::read(&model_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_model() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("test.scad");

        ModelStore::create(&model_path, "cube(10);").unwrap();
        let result = ModelStore::update(&model_path, "sphere(5);");

        assert!(result.is_ok());
        let updated = ModelStore::read(&model_path).unwrap();
        assert_eq!(updated, "sphere(5);");
    }

    #[test]
    fn test_update_nonexistent_model() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("nonexistent.scad");

        let result = ModelStore::update(&model_path, "cube(10);");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_model() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("test.scad");

        ModelStore::create(&model_path, "cube(10);").unwrap();
        assert!(model_path.exists());

        let result = ModelStore::delete(&model_path);
        assert!(result.is_ok());
        assert!(!model_path.exists());
    }

    #[test]
    fn test_delete_nonexistent_model() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("nonexistent.scad");

        let result = ModelStore::delete(&model_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_models_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = ModelStore::list_models(temp_dir.path()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_list_models_single() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("model.scad");

        ModelStore::create(&model_path, "cube(10);").unwrap();
        let result = ModelStore::list_models(temp_dir.path()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_name().unwrap(), "model.scad");
    }

    #[test]
    fn test_list_models_multiple() {
        let temp_dir = tempfile::tempdir().unwrap();

        ModelStore::create(temp_dir.path().join("model1.scad"), "cube(10);").unwrap();
        ModelStore::create(temp_dir.path().join("model2.scad"), "sphere(5);").unwrap();
        ModelStore::create(temp_dir.path().join("model3.scad"), "cylinder(h=10, r=5);").unwrap();

        let result = ModelStore::list_models(temp_dir.path()).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_list_models_filters_non_scad() {
        let temp_dir = tempfile::tempdir().unwrap();

        ModelStore::create(temp_dir.path().join("model.scad"), "cube(10);").unwrap();
        fs::write(temp_dir.path().join("readme.txt"), "Notes").unwrap();
        fs::write(temp_dir.path().join("data.json"), "{}").unwrap();

        let result = ModelStore::list_models(temp_dir.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_name().unwrap(), "model.scad");
    }

    #[test]
    fn test_list_models_nonexistent_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent");

        let result = ModelStore::list_models(&nonexistent).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_atomic_write_consistency() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("test.scad");
        let content = "cube([1, 2, 3]); sphere(r=5); cylinder(h=10, r=5);";

        ModelStore::create(&model_path, content).unwrap();
        let read_back = ModelStore::read(&model_path).unwrap();

        assert_eq!(read_back, content);
    }
}
